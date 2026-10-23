//! In-process Rust-owned MLX execution. No interpreter, subprocess, or serialization.
use super::completion_ledger::{Footprint, Lease, Ledger, Snapshot};
use super::weights::{Manifest, ModelWeights};
use anyhow::{bail, ensure, Context, Result};
use std::{
    ffi::{c_char, c_void, CStr, CString},
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    thread::JoinHandle,
};
use tokio::sync::{mpsc, oneshot, OwnedSemaphorePermit, Semaphore};

const MAX_NODES: usize = 256;
const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;
const MAX_GRAPH_BYTES: usize = 256 * 1024 * 1024;
pub type NodeId = u32;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    Cpu,
    Gpu,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Data {
    F32(Vec<f32>),
    I32(Vec<i32>),
    U32(Vec<u32>),
}
impl Data {
    fn len(&self) -> usize {
        match self {
            Self::F32(v) => v.len(),
            Self::I32(v) => v.len(),
            Self::U32(v) => v.len(),
        }
    }
    fn dtype(&self) -> i32 {
        match self {
            Self::F32(_) => 0,
            Self::I32(_) => 1,
            Self::U32(_) => 2,
        }
    }
    fn pointer(&self) -> *const c_void {
        match self {
            Self::F32(v) => v.as_ptr().cast(),
            Self::I32(v) => v.as_ptr().cast(),
            Self::U32(v) => v.as_ptr().cast(),
        }
    }
    fn mutable_pointer(&mut self) -> *mut c_void {
        match self {
            Self::F32(v) => v.as_mut_ptr().cast(),
            Self::I32(v) => v.as_mut_ptr().cast(),
            Self::U32(v) => v.as_mut_ptr().cast(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Tensor {
    pub shape: Vec<i32>,
    pub data: Data,
}
#[derive(Debug)]
enum Node {
    Input(Data),
    Parameter(CString),
    Kernel {
        code: i32,
        operands: Vec<NodeId>,
        ints: Vec<i32>,
        floats: Vec<f32>,
    },
    KernelOutput,
    Op {
        code: i32,
        operands: Vec<NodeId>,
        group: i32,
        epsilon: f32,
    },
}
#[derive(Clone, Debug)]
struct Descriptor {
    shape: Vec<i32>,
    dtype: i32,
    elements: usize,
}
#[derive(Default, Debug)]
pub struct Graph {
    nodes: Vec<Node>,
    descriptors: Vec<Descriptor>,
    input_bytes: usize,
    graph_bytes: usize,
    parameter_owner: Option<u64>,
}
impl Graph {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn parameter(&mut self, model: &ModelWeights, name: &str) -> Result<NodeId> {
        ensure!(
            self.parameter_owner
                .is_none_or(|owner| owner == model.owner),
            "mixed checkpoint/executor graph"
        );
        let info = model
            .manifest
            .parameters
            .get(name)
            .context("parameter not present in checkpoint")?;
        ensure!(self.nodes.len() < MAX_NODES, "node cap");
        let name = CString::new(name)?;
        let elements = info.shape.iter().map(|n| *n as usize).product();
        let id = self.nodes.len() as NodeId;
        self.nodes.push(Node::Parameter(name));
        self.descriptors.push(Descriptor {
            shape: info.shape.clone(),
            dtype: info.dtype,
            elements,
        });
        self.parameter_owner = Some(model.owner);
        Ok(id)
    }
    /// Bind the exact module's packed weights and quantization override.
    pub fn linear(&mut self, x: NodeId, model: &ModelWeights, module: &str) -> Result<NodeId> {
        let quant = model.manifest.module_quantization(module);
        let weight = self.parameter(model, &format!("{module}.weight"))?;
        let scales = self.parameter(model, &format!("{module}.scales"))?;
        let biases = self.parameter(model, &format!("{module}.biases"))?;
        self.quantized_matmul(x, weight, scales, biases, quant.group, quant.bits)
    }
    pub fn input(&mut self, shape: &[i32], data: Data) -> Result<NodeId> {
        ensure!(
            !shape.is_empty() && shape.len() <= 4 && shape.iter().all(|n| (1..=8192).contains(n)),
            "rank/dimension bounds"
        );
        let elements = shape
            .iter()
            .try_fold(1usize, |a, b| a.checked_mul(*b as usize))
            .context("shape overflow")?;
        ensure!(elements == data.len(), "input shape/data mismatch");
        ensure!(
            self.input_bytes
                .checked_add(elements * 4)
                .is_some_and(|b| b <= MAX_INPUT_BYTES),
            "input byte cap"
        );
        if let Data::F32(values) = &data {
            ensure!(values.iter().all(|v| v.is_finite()), "nonfinite input");
        }
        let dtype = data.dtype();
        let id = self.push(
            Node::Input(data),
            Descriptor {
                shape: shape.to_vec(),
                dtype,
                elements,
            },
        )?;
        self.input_bytes += elements * 4;
        Ok(id)
    }
    fn descriptor(&self, id: NodeId) -> Result<&Descriptor> {
        self.descriptors
            .get(id as usize)
            .context("invalid node reference")
    }
    fn push(&mut self, node: Node, descriptor: Descriptor) -> Result<NodeId> {
        ensure!(self.nodes.len() < MAX_NODES, "node cap");
        let bytes = descriptor
            .elements
            .checked_mul(4)
            .context("tensor byte overflow")?;
        let total = self
            .graph_bytes
            .checked_add(bytes)
            .filter(|n| *n <= MAX_GRAPH_BYTES)
            .context("logical graph byte cap")?;
        let id = self.nodes.len() as NodeId;
        self.nodes.push(node);
        self.descriptors.push(descriptor);
        self.graph_bytes = total;
        Ok(id)
    }
    fn binary(&mut self, code: i32, a: NodeId, b: NodeId) -> Result<NodeId> {
        let left = self.descriptor(a)?;
        let right = self.descriptor(b)?;
        ensure!(
            left.shape == right.shape && left.dtype == right.dtype,
            "elementwise shape/dtype mismatch"
        );
        self.push(
            Node::Op {
                code,
                operands: vec![a, b],
                group: 0,
                epsilon: 0.0,
            },
            left.clone(),
        )
    }
    pub fn add(&mut self, a: NodeId, b: NodeId) -> Result<NodeId> {
        self.binary(0, a, b)
    }
    pub fn multiply(&mut self, a: NodeId, b: NodeId) -> Result<NodeId> {
        self.binary(1, a, b)
    }
    pub fn matmul(&mut self, a: NodeId, b: NodeId) -> Result<NodeId> {
        let left = self.descriptor(a)?;
        let right = self.descriptor(b)?;
        ensure!(
            left.dtype == 0
                && right.dtype == 0
                && left.shape.len() == 2
                && right.shape.len() == 2
                && left.shape[1] == right.shape[0],
            "F32 rank-2 matmul shape/dtype mismatch"
        );
        let shape = vec![left.shape[0], right.shape[1]];
        let elements = shape[0] as usize * shape[1] as usize;
        self.push(
            Node::Op {
                code: 2,
                operands: vec![a, b],
                group: 0,
                epsilon: 0.0,
            },
            Descriptor {
                shape,
                dtype: 0,
                elements,
            },
        )
    }
    /// Affine Q4: packed [N,K/8], scales and biases [N,K/group], input [M,K].
    pub fn q4_matmul(
        &mut self,
        x: NodeId,
        packed: NodeId,
        scales: NodeId,
        biases: NodeId,
        group: i32,
    ) -> Result<NodeId> {
        self.quantized_matmul(x, packed, scales, biases, group, 4)
    }
    pub fn quantized_matmul(
        &mut self,
        x: NodeId,
        packed: NodeId,
        scales: NodeId,
        biases: NodeId,
        group: i32,
        bits: i32,
    ) -> Result<NodeId> {
        ensure!(
            [32, 64, 128].contains(&group) && [2, 3, 4, 5, 6, 8].contains(&bits),
            "unsupported affine quantization"
        );
        let a = self.descriptor(x)?;
        let w = self.descriptor(packed)?;
        let s = self.descriptor(scales)?;
        let b = self.descriptor(biases)?;
        ensure!(
            a.dtype == 0
                && w.dtype == 2
                && [0, 3, 4].contains(&s.dtype)
                && s.dtype == b.dtype
                && a.shape.len() == 2
                && w.shape.len() == 2
                && s.shape.len() == 2
                && s.shape == b.shape,
            "Q4 dtype/rank mismatch"
        );
        let k = a.shape[1];
        ensure!(
            k % group == 0
                && w.shape[1] as i64 == k as i64 * bits as i64 / 32
                && s.shape == [w.shape[0], k / group],
            "Q4 packed shape mismatch"
        );
        let shape = vec![a.shape[0], w.shape[0]];
        let elements = shape[0] as usize * shape[1] as usize;
        self.push(
            Node::Op {
                code: 100 + bits,
                operands: vec![x, packed, scales, biases],
                group,
                epsilon: 0.0,
            },
            Descriptor {
                shape,
                dtype: 0,
                elements,
            },
        )
    }
    pub fn rms_norm(&mut self, x: NodeId, weight: NodeId, epsilon: f32) -> Result<NodeId> {
        let a = self.descriptor(x)?;
        let w = self.descriptor(weight)?;
        ensure!(
            a.dtype == 0
                && [0, 3, 4].contains(&w.dtype)
                && w.shape == [*a.shape.last().unwrap()]
                && epsilon.is_finite()
                && epsilon > 0.0,
            "RMS shape/dtype/epsilon mismatch"
        );
        self.push(
            Node::Op {
                code: 4,
                operands: vec![x, weight],
                group: 0,
                epsilon,
            },
            a.clone(),
        )
    }
}
#[derive(Default)]
struct Counters {
    submitted: AtomicUsize,
    completed: AtomicUsize,
    failed: AtomicUsize,
    abandoned: AtomicUsize,
    active: AtomicUsize,
    peak: AtomicUsize,
    closed: AtomicBool,
    poisoned: AtomicBool,
    parameter_bytes: AtomicUsize,
}
#[derive(Debug)]
pub struct Stats {
    pub submitted: usize,
    pub completed: usize,
    pub failed: usize,
    pub abandoned: usize,
    pub in_flight: usize,
    pub peak_in_flight: usize,
    pub admission_closed: bool,
    pub resources: Snapshot,
    pub parameter_bytes: usize,
}
struct Command {
    graph: Graph,
    output: NodeId,
    reply: oneshot::Sender<Result<Tensor>>,
    permit: OwnedSemaphorePermit,
    lease: Lease,
}
pub struct Backend {
    tx: Option<mpsc::Sender<Command>>,
    permits: Arc<Semaphore>,
    counters: Arc<Counters>,
    thread: Option<JoinHandle<()>>,
    ledger: Ledger,
    owner: u64,
    model: Option<ModelWeights>,
}
pub struct Receipt(oneshot::Receiver<Result<Tensor>>);
impl Receipt {
    pub async fn result(self) -> Result<Tensor> {
        self.0
            .await
            .context("native worker stopped before receipt")?
    }
}
impl Backend {
    pub fn new(device: Device) -> Result<Self> {
        Self::with_limits(
            device,
            Footprint {
                host_bytes: 192 * 1024 * 1024,
                tensor_bytes: 384 * 1024 * 1024,
                graph_nodes: 512,
            },
        )
    }
    pub fn with_limits(device: Device, limits: Footprint) -> Result<Self> {
        Self::start(device, limits, None)
    }
    pub async fn with_checkpoint(device: Device, directory: std::path::PathBuf) -> Result<Self> {
        tokio::task::spawn_blocking(move || {
            let manifest = Arc::new(Manifest::read(&directory)?);
            Self::start(
                device,
                Footprint {
                    host_bytes: 192 * 1024 * 1024,
                    tensor_bytes: 384 * 1024 * 1024,
                    graph_nodes: 512,
                },
                Some(manifest),
            )
        })
        .await?
    }
    pub fn model(&self) -> Option<&ModelWeights> {
        self.model.as_ref()
    }
    fn start(device: Device, limits: Footprint, manifest: Option<Arc<Manifest>>) -> Result<Self> {
        static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
        let owner = NEXT_OWNER.fetch_add(1, Ordering::Relaxed);
        let model = manifest.as_ref().map(|m| ModelWeights {
            owner,
            manifest: m.clone(),
        });
        let ledger = Ledger::new(limits)?;
        let (tx, mut rx) = mpsc::channel::<Command>(3);
        let counters = Arc::new(Counters::default());
        let shared = counters.clone();
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let thread = std::thread::Builder::new()
            .name("rust-mlx-native".into())
            .spawn(move || {
                let weights = match manifest
                    .as_ref()
                    .map(|m| NativeWeights::open(m))
                    .transpose()
                {
                    Ok(weights) => weights,
                    Err(error) => {
                        shared.closed.store(true, Ordering::Release);
                        let _ = ready_tx.send(Err(error));
                        return;
                    }
                };
                shared.parameter_bytes.store(
                    manifest.as_ref().map_or(0, |m| m.payload_bytes),
                    Ordering::Release,
                );
                let _ = ready_tx.send(Ok(()));
                while let Some(command) = rx.blocking_recv() {
                    let Command {
                        graph,
                        output,
                        reply,
                        permit,
                        lease,
                    } = command;
                    let result = if shared.poisoned.load(Ordering::Acquire) {
                        drop(graph);
                        Err(anyhow::anyhow!("native executor latched closed"))
                    } else {
                        let uses_parameters = graph.parameter_owner.is_some();
                        (|| {
                            if uses_parameters {
                                manifest
                                    .as_ref()
                                    .context("checkpoint missing")?
                                    .verify_stamps()?;
                            }
                            let result = execute(graph, output, device, weights.as_ref());
                            if uses_parameters {
                                manifest
                                    .as_ref()
                                    .context("checkpoint missing")?
                                    .verify_stamps()?;
                            }
                            result
                        })()
                    };
                    if result.is_ok() {
                        shared.completed.fetch_add(1, Ordering::Relaxed);
                    } else {
                        shared.failed.fetch_add(1, Ordering::Relaxed);
                        shared.poisoned.store(true, Ordering::Release);
                        shared.closed.store(true, Ordering::Release);
                    }
                    shared.active.fetch_sub(1, Ordering::AcqRel);
                    drop(lease);
                    drop(permit); // Native graph was destroyed and stream synchronized before releasing admission.
                    if reply.send(result).is_err() {
                        shared.abandoned.fetch_add(1, Ordering::Relaxed);
                    }
                }
                shared.closed.store(true, Ordering::Release);
                drop(weights);
                shared.parameter_bytes.store(0, Ordering::Release);
                // All graph owners are gone; release this thread's MLX streams.
                unsafe {
                    rs_mlx_cleanup();
                }
            })?;
        if let Err(error) = ready_rx.recv().context("native initializer stopped")? {
            let _ = thread.join();
            return Err(error);
        }
        Ok(Self {
            tx: Some(tx),
            permits: Arc::new(Semaphore::new(3)),
            counters,
            thread: Some(thread),
            ledger,
            owner,
            model,
        })
    }
    /// Nonblocking bounded admission. Caller retries/backpressures; no unbounded waiter queue.
    pub fn submit(&self, graph: Graph, output: NodeId) -> Result<Receipt> {
        let descriptor = graph.descriptor(output)?;
        ensure!(
            graph
                .parameter_owner
                .is_none_or(|owner| owner == self.owner),
            "parameter reference belongs to another executor"
        );
        ensure!(
            !self.counters.closed.load(Ordering::Acquire),
            "native admission closed"
        );
        let permit = self
            .permits
            .clone()
            .try_acquire_owned()
            .context("native in-flight limit reached")?;
        let lease = self.ledger.reserve(Footprint {
            host_bytes: graph.input_bytes + descriptor.elements * 4,
            tensor_bytes: graph.graph_bytes,
            graph_nodes: graph.nodes.len(),
        })?;
        let (reply, receive) = oneshot::channel();
        let tx = self.tx.as_ref().context("native backend shut down")?;
        let active = self.counters.active.fetch_add(1, Ordering::AcqRel) + 1;
        self.counters.submitted.fetch_add(1, Ordering::Relaxed);
        if let Err(error) = tx.try_send(Command {
            graph,
            output,
            reply,
            permit,
            lease,
        }) {
            self.counters.active.fetch_sub(1, Ordering::AcqRel);
            self.counters.submitted.fetch_sub(1, Ordering::Relaxed);
            bail!("native submission failed: {error}");
        }
        self.counters.peak.fetch_max(active, Ordering::Relaxed);
        Ok(Receipt(receive))
    }
    pub fn stats(&self) -> Stats {
        let c = &self.counters;
        Stats {
            submitted: c.submitted.load(Ordering::Acquire),
            completed: c.completed.load(Ordering::Acquire),
            failed: c.failed.load(Ordering::Acquire),
            abandoned: c.abandoned.load(Ordering::Acquire),
            in_flight: c.active.load(Ordering::Acquire),
            peak_in_flight: c.peak.load(Ordering::Acquire),
            admission_closed: c.closed.load(Ordering::Acquire),
            resources: self.ledger.snapshot(),
            parameter_bytes: c.parameter_bytes.load(Ordering::Acquire),
        }
    }
    pub async fn shutdown(mut self) -> Result<Stats> {
        self.counters.closed.store(true, Ordering::Release);
        // Drain accepted work: closing admission must not poison already-accepted commands.
        self.tx.take();
        if let Some(thread) = self.thread.take() {
            tokio::task::spawn_blocking(move || {
                thread
                    .join()
                    .map_err(|_| anyhow::anyhow!("native executor panicked"))
            })
            .await??;
        }
        Ok(self.stats())
    }
}
impl Drop for Backend {
    fn drop(&mut self) {
        self.tx.take(); /* Detached executor retains accepted commands until completion. Explicit shutdown joins. */
    }
}

unsafe extern "C" {
    fn rs_mlx_kernel(
        graph: *mut c_void,
        op: i32,
        inputs: *const u32,
        count: usize,
        ints: *const i32,
        ni: usize,
        floats: *const f32,
        nf: usize,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn rs_mlx_create(gpu: i32, error: *mut c_char, capacity: usize) -> *mut c_void;
    fn rs_mlx_destroy(graph: *mut c_void);
    fn rs_mlx_input(
        graph: *mut c_void,
        dtype: i32,
        data: *const c_void,
        shape: *const i32,
        rank: usize,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn rs_mlx_op(
        graph: *mut c_void,
        op: i32,
        inputs: *const u32,
        count: usize,
        group: i32,
        epsilon: f32,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn rs_mlx_finish(
        graph: *mut c_void,
        output: u32,
        data: *mut c_void,
        elements: usize,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn rs_mlx_version() -> *const c_char;
    fn rs_mlx_cleanup();
    fn rs_mlx_weights_create(error: *mut c_char, capacity: usize) -> *mut c_void;
    fn rs_mlx_weights_destroy(store: *mut c_void);
    fn rs_mlx_weights_load(
        store: *mut c_void,
        file: *const c_char,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn rs_mlx_parameter(
        graph: *mut c_void,
        store: *mut c_void,
        name: *const c_char,
        shape: *const i32,
        rank: usize,
        dtype: i32,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
}
pub(super) struct NativeWeights(pub(super) *mut c_void);
impl NativeWeights {
    pub(super) fn open_mtp(
        manifest: &Manifest,
        sidecar: &super::native_mtp::MtpSidecar,
    ) -> Result<Self> {
        manifest.verify_stamps()?;
        ensure!(
            sidecar.identity == manifest.identity
                && sidecar.path == manifest.directory.join("model-mtp.safetensors"),
            "foreign MTP sidecar"
        );
        let mut error = [0 as c_char; 1024];
        let pointer = unsafe { rs_mlx_weights_create(error.as_mut_ptr(), error.len()) };
        if pointer.is_null() {
            status(-1, &error)?;
            unreachable!();
        }
        let store = Self(pointer);
        let path = CString::new(sidecar.path.to_str().context("non-UTF8 MTP path")?)?;
        status(
            unsafe { rs_mlx_weights_load(store.0, path.as_ptr(), error.as_mut_ptr(), error.len()) },
            &error,
        )?;
        manifest.verify_stamps()?;
        Ok(store)
    }
    pub(super) fn open(manifest: &Manifest) -> Result<Self> {
        manifest.verify_stamps()?;
        let mut error = [0 as c_char; 1024];
        let pointer = unsafe { rs_mlx_weights_create(error.as_mut_ptr(), error.len()) };
        if pointer.is_null() {
            status(-1, &error)?;
            unreachable!();
        }
        let store = Self(pointer);
        for shard in &manifest.shards {
            let name = CString::new(shard.to_str().context("non-UTF8 shard path")?)?;
            let code = unsafe {
                rs_mlx_weights_load(store.0, name.as_ptr(), error.as_mut_ptr(), error.len())
            };
            status(code, &error)?;
        }
        manifest.verify_stamps()?;
        Ok(store)
    }
}
impl Drop for NativeWeights {
    fn drop(&mut self) {
        unsafe { rs_mlx_weights_destroy(self.0) }
    }
}
struct Native(*mut c_void);
impl Drop for Native {
    fn drop(&mut self) {
        unsafe { rs_mlx_destroy(self.0) }
    }
}
fn status(code: i32, error: &[c_char; 1024]) -> Result<()> {
    if code != 0 {
        // Bridge guarantees NUL termination; buffer is zero initialized as a second guard.
        let text = unsafe { CStr::from_ptr(error.as_ptr()) }.to_string_lossy();
        bail!("native MLX: {text}");
    }
    Ok(())
}
fn execute(
    graph: Graph,
    output: NodeId,
    device: Device,
    weights: Option<&NativeWeights>,
) -> Result<Tensor> {
    let mut error = [0 as c_char; 1024];
    let pointer = unsafe {
        rs_mlx_create(
            i32::from(device == Device::Gpu),
            error.as_mut_ptr(),
            error.len(),
        )
    };
    if pointer.is_null() {
        status(-1, &error)?;
        unreachable!();
    }
    let native = Native(pointer);
    for (node, descriptor) in graph.nodes.iter().zip(&graph.descriptors) {
        let code = unsafe {
            match node {
                Node::Input(data) => rs_mlx_input(
                    native.0,
                    data.dtype(),
                    data.pointer(),
                    descriptor.shape.as_ptr(),
                    descriptor.shape.len(),
                    error.as_mut_ptr(),
                    error.len(),
                ),
                Node::Parameter(name) => rs_mlx_parameter(
                    native.0,
                    weights.context("parameter store unavailable")?.0,
                    name.as_ptr(),
                    descriptor.shape.as_ptr(),
                    descriptor.shape.len(),
                    descriptor.dtype,
                    error.as_mut_ptr(),
                    error.len(),
                ),
                Node::Kernel {
                    code,
                    operands,
                    ints,
                    floats,
                } => rs_mlx_kernel(
                    native.0,
                    *code,
                    operands.as_ptr(),
                    operands.len(),
                    ints.as_ptr(),
                    ints.len(),
                    floats.as_ptr(),
                    floats.len(),
                    error.as_mut_ptr(),
                    error.len(),
                ),
                Node::KernelOutput => 0,
                Node::Op {
                    code,
                    operands,
                    group,
                    epsilon,
                } => rs_mlx_op(
                    native.0,
                    *code,
                    operands.as_ptr(),
                    operands.len(),
                    *group,
                    *epsilon,
                    error.as_mut_ptr(),
                    error.len(),
                ),
            }
        };
        status(code, &error)?;
    }
    let descriptor = graph.descriptor(output)?;
    let mut data = match descriptor.dtype {
        0 | 3 | 4 => Data::F32(vec![0.0; descriptor.elements]),
        1 => Data::I32(vec![0; descriptor.elements]),
        _ => Data::U32(vec![0; descriptor.elements]),
    };
    let code = unsafe {
        rs_mlx_finish(
            native.0,
            output,
            data.mutable_pointer(),
            descriptor.elements,
            error.as_mut_ptr(),
            error.len(),
        )
    };
    status(code, &error)?;
    if let Data::F32(values) = &data {
        ensure!(
            values.iter().all(|v| v.is_finite()),
            "native result nonfinite"
        );
    }
    Ok(Tensor {
        shape: descriptor.shape.clone(),
        data,
    })
}
pub fn version() -> String {
    unsafe { CStr::from_ptr(rs_mlx_version()) }
        .to_string_lossy()
        .into_owned()
}
pub async fn smoke() -> Result<()> {
    let backend = Backend::new(Device::Gpu)?;
    let mut graph = Graph::new();
    let a = graph.input(&[2, 2], Data::F32(vec![1.0, 2.0, 3.0, 4.0]))?;
    let b = graph.input(&[2, 2], Data::F32(vec![2.0, 0.0, 1.0, 2.0]))?;
    let output = graph.matmul(a, b)?;
    let result = backend.submit(graph, output)?.result().await?;
    ensure!(
        result.data == Data::F32(vec![4.0, 4.0, 10.0, 8.0]),
        "native matmul oracle mismatch"
    );
    let stats = backend.shutdown().await?;
    println!(
        "Native Rust MLX {} | GPU | Python: disabled | JSON transport: none",
        version()
    );
    println!("Tensor smoke passed: {:?}; completed={} failed={} in_flight={}; Qwen qualification: run native-qualify",result.data,stats.completed,stats.failed,stats.in_flight);
    Ok(())
}

/// Macro throughput includes Rust admission, graph construction, native execution,
/// completion and CPU-owned result transfer. It is not model-token throughput.
pub async fn benchmark() -> Result<()> {
    println!(
        "NATIVE-MLX-TENSOR-BENCH | MLX {} | GPU | 3 admitted graphs | no Python/JSON",
        version()
    );
    println!("size window graphs_per_second elapsed_ms peak_admitted peak_host_bytes peak_tensor_bytes peak_nodes");
    for size in [64, 128, 256, 512] {
        for window in [1000, 1500, 2000] {
            let backend = Backend::new(Device::Gpu)?;
            let make = || -> Result<(Graph, NodeId)> {
                let mut graph = Graph::new();
                let x = graph.input(
                    &[size, size],
                    Data::F32(vec![0.125; (size * size) as usize]),
                )?;
                let w =
                    graph.input(&[size, size], Data::F32(vec![0.25; (size * size) as usize]))?;
                let y = graph.matmul(x, w)?;
                Ok((graph, y))
            };
            let (g, y) = make()?;
            let warm = backend.submit(g, y)?.result().await?;
            if let Data::F32(values) = warm.data {
                ensure!(
                    values
                        .iter()
                        .all(|v| (*v - size as f32 / 32.0).abs() < 1e-5),
                    "warmup arithmetic oracle"
                );
            }
            let start = std::time::Instant::now();
            for base in (0..window).step_by(3) {
                let mut receipts = Vec::with_capacity(3);
                for _ in base..(base + 3).min(window) {
                    let (g, y) = make()?;
                    receipts.push(backend.submit(g, y)?);
                }
                for receipt in receipts {
                    let _ = receipt.result().await?;
                }
            }
            let seconds = start.elapsed().as_secs_f64();
            let stats = backend.shutdown().await?;
            ensure!(
                stats.completed == window + 1
                    && stats.failed == 0
                    && stats.resources.leases == 0
                    && stats.resources.used == Footprint::default(),
                "benchmark drain accounting"
            );
            println!(
                "{size} {window} {:.3} {:.3} {} {} {} {}",
                window as f64 / seconds,
                seconds * 1000.0,
                stats.peak_in_flight,
                stats.resources.peak.host_bytes,
                stats.resources.peak.tensor_bytes,
                stats.resources.peak.graph_nodes
            );
        }
    }
    Ok(())
}
#[path = "native_ops.rs"]
mod ops;

/// Parameter/projection qualification only: does not claim autoregressive generation.
pub async fn checkpoint_smoke() -> Result<()> {
    let path = std::path::PathBuf::from(include_str!("../../fixtures/native-model.path").trim());
    let backend = Backend::with_checkpoint(Device::Gpu, path).await?;
    let model = backend.model().context("missing checkpoint")?;
    println!("NATIVE-WEIGHTS identity={} parameters={} payload_bytes={} mtp_present={} mtp_executed=false", model.manifest.identity, model.manifest.parameters.len(), model.manifest.payload_bytes, model.manifest.mtp_present);
    for module in [
        "language_model.model.layers.0.linear_attn.in_proj_a",
        "language_model.model.layers.0.mlp.gate",
    ] {
        let quant = model.manifest.module_quantization(module);
        let scale = model
            .manifest
            .parameters
            .get(&format!("{module}.scales"))
            .context("projection scales missing")?;
        let k = scale.shape[1] * quant.group;
        let mut previous = None;
        for _ in 0..2 {
            let mut graph = Graph::new();
            let x = graph.input(
                &[1, k],
                Data::F32((0..k).map(|i| (i % 7 - 3) as f32 / 16.).collect()),
            )?;
            let y = graph.linear(x, model, module)?;
            let data = backend.submit(graph, y)?.result().await?.data;
            if let Some(ref expected) = previous {
                ensure!(&data == expected, "persistent projection repeat mismatch");
            }
            previous = Some(data);
        }
        println!(
            "NATIVE-PROJECTION module={module} bits={} group={} repeat=exact",
            quant.bits, quant.group
        );
    }
    let stats = backend.shutdown().await?;
    ensure!(
        stats.parameter_bytes == 0 && stats.resources.leases == 0,
        "checkpoint shutdown accounting"
    );
    println!("NATIVE-WEIGHTS drained=true generation_qualified=false");
    Ok(())
}
impl Backend {
    pub(crate) fn owner_id(&self) -> u64 {
        self.owner
    }
}
