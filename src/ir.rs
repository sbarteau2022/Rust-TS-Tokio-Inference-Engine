use crate::wire::Op;
use anyhow::{bail, ensure, Result};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Serialize)]
pub struct Tile {
    pub rows: u32,
    pub cols: u32,
    pub uses: u32,
    pub tile_uses: u32,
    pub layout: &'static str,
    pub residency: &'static str,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelKind {
    GemmBiasRelu,
    AddRelu,
    Sin,
    Cos,
    Atan2,
    Tanh,
}
impl KernelKind {
    pub fn is_cordic(self) -> bool {
        matches!(self, Self::Sin | Self::Cos | Self::Atan2 | Self::Tanh)
    }
    pub fn entry(self) -> &'static str {
        match self {
            Self::GemmBiasRelu => "mvk",
            Self::AddRelu => "add_relu",
            Self::Sin => "cordic_sin",
            Self::Cos => "cordic_cos",
            Self::Atan2 => "cordic_atan2",
            Self::Tanh => "cordic_tanh",
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    pub kind: KernelKind,
    pub m: u32,
    pub k: u32,
    pub n: u32,
    pub inputs: Vec<String>,
    pub input_values: Vec<u32>,
    pub output: String,
    pub tiles: BTreeMap<u32, Tile>,
}
fn name_ok(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.as_bytes()[0].is_ascii_alphabetic()
        && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
}
pub fn lower(ops: &[Op]) -> Result<Plan> {
    if matches!(ops.len(), 3 | 4) {
        return lower_cordic(ops);
    }
    if ops.len() == 5 {
        return lower_elementwise(ops);
    }
    let mut tiles: BTreeMap<u32, Tile> = BTreeMap::new();
    let mut names = BTreeSet::new();
    for op in ops {
        let (out, shape, uses) = match op {
            Op::Load {
                name,
                rows,
                cols,
                out,
            } => {
                ensure!(
                    name_ok(name) && names.insert(name),
                    "invalid or duplicate name"
                );
                ensure!(
                    *rows > 0 && *cols > 0 && *rows <= 4096 && *cols <= 4096,
                    "shape budget"
                );
                (Some(*out), (*rows, *cols), vec![])
            }
            Op::Dot { a, b, out } | Op::Add { a, b, out } => {
                let x = tiles
                    .get(a)
                    .ok_or_else(|| anyhow::anyhow!("undefined SSA value"))?;
                let y = tiles
                    .get(b)
                    .ok_or_else(|| anyhow::anyhow!("undefined SSA value"))?;
                match op {
                    Op::Dot { .. } => ensure!(x.cols == y.rows, "dot dimension mismatch"),
                    _ => ensure!(
                        x.cols == y.cols && (y.rows == 1 || y.rows == x.rows),
                        "add dimension mismatch"
                    ),
                };
                (
                    Some(*out),
                    (
                        x.rows,
                        if matches!(op, Op::Dot { .. }) {
                            y.cols
                        } else {
                            x.cols
                        },
                    ),
                    vec![*a, *b],
                )
            }
            Op::Relu { a, out } => {
                let x = tiles
                    .get(a)
                    .ok_or_else(|| anyhow::anyhow!("undefined SSA value"))?;
                (Some(*out), (x.rows, x.cols), vec![*a])
            }
            Op::Cordic { .. } => bail!("CORDIC cannot appear in GEMM graph"),
            Op::Store { a, name } => {
                ensure!(
                    name_ok(name) && names.insert(name),
                    "invalid or aliasing output name"
                );
                (None, (0, 0), vec![*a])
            }
        };
        for id in uses {
            tiles
                .get_mut(&id)
                .ok_or_else(|| anyhow::anyhow!("undefined SSA value"))?
                .uses += 1;
        }
        if let Some(id) = out {
            ensure!(!tiles.contains_key(&id), "SSA redefinition");
            tiles.insert(
                id,
                Tile {
                    rows: shape.0,
                    cols: shape.1,
                    uses: 0,
                    tile_uses: 0,
                    layout: "row_major_q1_31",
                    residency: "register",
                },
            );
        }
    }
    // Only this exact dataflow is supported; never silently ignore trace operations.
    let [Op::Load {
        name: an,
        rows: m,
        cols: k,
        out: a,
    }, Op::Load {
        name: bn,
        rows: bk,
        cols: n,
        out: b,
    }, Op::Load {
        name: cn,
        rows: cr,
        cols: cc,
        out: c,
    }, Op::Dot {
        a: da,
        b: db,
        out: d,
    }, Op::Add {
        a: aa,
        b: ab,
        out: e,
    }, Op::Relu { a: ra, out: r }, Op::Store { a: sa, name: sn }] = ops
    else {
        bail!("unsupported fusion graph")
    };
    ensure!(
        (a, b, d, c, e, r) == (da, db, aa, ab, ra, sa),
        "unsupported fusion edges"
    );
    ensure!(bk == k && *cr == 1 && cc == n, "fusion shape mismatch");
    ensure!(
        u64::from(*m) * u64::from(*k) * u64::from(*n) <= 536870912,
        "compute budget"
    );
    // Shared tiles feed 8 output lanes, even when the trace has only one dot consumer.
    for (id, t) in &mut tiles {
        t.tile_uses = t.uses * if *id == *a || *id == *b { 8 } else { 1 };
        t.residency = if t.tile_uses > 1 {
            "simdgroup_register"
        } else {
            "register"
        };
    }
    Ok(Plan {
        kind: KernelKind::GemmBiasRelu,
        m: *m,
        k: *k,
        n: *n,
        inputs: vec![an.clone(), bn.clone(), cn.clone()],
        input_values: vec![*a, *b, *c],
        output: sn.clone(),
        tiles,
    })
}
impl Plan {
    pub fn input_count(&self) -> usize {
        match self.kind {
            KernelKind::GemmBiasRelu => 3,
            KernelKind::AddRelu | KernelKind::Atan2 => 2,
            KernelKind::Sin | KernelKind::Cos | KernelKind::Tanh => 1,
        }
    }
    pub fn input_shapes(&self) -> Vec<(u32, u32)> {
        match self.kind {
            KernelKind::GemmBiasRelu => vec![(self.m, self.k), (self.k, self.n), (1, self.n)],
            KernelKind::AddRelu => vec![(self.m, self.n), (1, self.n)],
            KernelKind::Atan2 => vec![(self.m, self.n); 2],
            KernelKind::Sin | KernelKind::Cos | KernelKind::Tanh => vec![(self.m, self.n)],
        }
    }
    pub fn input_lengths(&self) -> Vec<usize> {
        self.input_shapes()
            .iter()
            .map(|(r, c)| *r as usize * *c as usize)
            .collect()
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            [self.m, self.k, self.n]
                .iter()
                .all(|x| (1..=4096).contains(x)),
            "plan shape budget"
        );
        ensure!(
            u64::from(self.m) * u64::from(self.k) * u64::from(self.n) <= 536870912,
            "plan compute budget"
        );
        ensure!(
            self.kind == KernelKind::GemmBiasRelu || self.k == 1,
            "elementwise K must be one"
        );
        let names: BTreeSet<_> = self.inputs.iter().collect();
        let ids: BTreeSet<_> = self.input_values.iter().collect();
        ensure!(
            names.len() == self.input_count()
                && ids.len() == self.input_count()
                && self.inputs.len() == self.input_count()
                && self.input_values.len() == self.input_count()
                && self.inputs.iter().all(|x| name_ok(x))
                && name_ok(&self.output)
                && !self.inputs.contains(&self.output),
            "plan bindings"
        );
        let shapes = self.input_shapes();
        for (id, shape) in self.input_values.iter().zip(shapes) {
            let tile = self
                .tiles
                .get(id)
                .ok_or_else(|| anyhow::anyhow!("missing plan input"))?;
            ensure!(
                (tile.rows, tile.cols) == shape && tile.layout == "row_major_q1_31",
                "plan input layout mismatch"
            );
        }
        Ok(())
    }
    pub fn msl(&self) -> Result<String> {
        self.validate()?;
        if self.kind.is_cordic() {
            return Ok(crate::cordic::msl(self.kind, self.m * self.n));
        }
        let helpers = r#"#include <metal_stdlib>
using namespace metal;
// Q1.31 floor rescale: explicit sign extension avoids signed-shift dependence.
inline int64_t qmul(int32_t a, int32_t b) {
 int64_t product=int64_t(a)*int64_t(b);
 uint64_t bits=as_type<uint64_t>(product);
 uint64_t sign=product<0 ? (~uint64_t(0) << 33) : uint64_t(0);
 return as_type<int64_t>((bits >> 31) | sign);
}
inline int32_t finish(int64_t value) { return int32_t(clamp(value, int64_t(0), int64_t(2147483647))); }
"#;
        if self.kind == KernelKind::AddRelu {
            return Ok(format!("{helpers}kernel void add_relu(device const int32_t* A [[buffer(0)]], device const int32_t* bias [[buffer(1)]], device int32_t* C [[buffer(2)]], uint i [[thread_position_in_grid]]) {{ if(i < {count}) C[i]=finish(int64_t(A[i])+int64_t(bias[i%{n}])); }}",count=self.m*self.n,n=self.n));
        }
        for id in &self.input_values[..2] {
            let tile = self
                .tiles
                .get(id)
                .ok_or_else(|| anyhow::anyhow!("missing input layout"))?;
            ensure!(
                tile.layout == "row_major_q1_31" && tile.tile_uses > 1,
                "unsupported operand layout/use count"
            );
            ensure!(
                tile.residency == "simdgroup_register",
                "unsupported operand residency"
            );
        }
        Ok(format!(
            r#"{helpers}
kernel void mvk(device const int32_t* A [[buffer(0)]], device const int32_t* B [[buffer(1)]], device const int32_t* bias [[buffer(2)]], device int32_t* C [[buffer(3)]], uint3 group [[threadgroup_position_in_grid]], uint lane [[thread_index_in_simdgroup]], uint sg [[simdgroup_index_in_threadgroup]], uint3 width [[threads_per_threadgroup]]) {{
 const uint M={m}, K={k}, N={n};
 uint row=group.y, col=group.x*(width.x/32)+sg;
 if(row>=M || col>=N) return;
 int64_t acc=0;
 for(uint x=lane;x<K;x+=32) acc+=qmul(A[row*K+x],B[x*N+col]);
 // All 32 lanes participate; exchange both halves of each signed 64-bit partial.
 for(uint offset=16;offset>0;offset>>=1) {{
  uint64_t bits=as_type<uint64_t>(acc);
  uint32_t lo=simd_shuffle_down(uint32_t(bits),offset);
  uint32_t hi=simd_shuffle_down(uint32_t(bits>>32),offset);
  int64_t other=as_type<int64_t>((uint64_t(hi)<<32)|uint64_t(lo));
  if(lane<offset) acc+=other;
 }}
 if(lane==0) C[row*N+col]=finish(acc+int64_t(bias[col]));
}}
"#,
            m = self.m,
            k = self.k,
            n = self.n
        ))
    }
}

/// Borrowed FlatBuffers accessors feed SSA directly, with no owned wire-op list.
pub fn lower_trace(bytes: &[u8]) -> Result<Plan> {
    lower_verified(crate::wire::verified(bytes)?)
}
pub(crate) fn lower_verified(trace: crate::trace_generated::wire::Trace<'_>) -> Result<Plan> {
    let ops = trace.ops();
    if matches!(ops.len(), 3 | 4) {
        return lower_cordic_view(trace);
    }
    if ops.len() == 5 {
        return lower_elementwise_view(trace);
    }
    ensure!(ops.len() == 7, "unsupported fusion graph");
    let a = ops
        .get(0)
        .operation_as_load()
        .ok_or_else(|| anyhow::anyhow!("expected A load"))?;
    let b = ops
        .get(1)
        .operation_as_load()
        .ok_or_else(|| anyhow::anyhow!("expected B load"))?;
    let c = ops
        .get(2)
        .operation_as_load()
        .ok_or_else(|| anyhow::anyhow!("expected bias load"))?;
    let d = ops
        .get(3)
        .operation_as_dot()
        .ok_or_else(|| anyhow::anyhow!("expected dot"))?;
    let e = ops
        .get(4)
        .operation_as_add()
        .ok_or_else(|| anyhow::anyhow!("expected add"))?;
    let r = ops
        .get(5)
        .operation_as_relu()
        .ok_or_else(|| anyhow::anyhow!("expected relu"))?;
    let store = ops
        .get(6)
        .operation_as_store()
        .ok_or_else(|| anyhow::anyhow!("expected store"))?;
    let ids = [a.out(), b.out(), c.out(), d.out(), e.out(), r.out()];
    ensure!(
        ids.iter().collect::<BTreeSet<_>>().len() == 6,
        "SSA redefinition"
    );
    ensure!(
        (d.a(), d.b(), e.a(), e.b(), r.a(), store.a())
            == (a.out(), b.out(), d.out(), c.out(), e.out(), r.out()),
        "unsupported fusion edges"
    );
    ensure!(
        a.cols() == b.rows() && c.rows() == 1 && c.cols() == b.cols(),
        "fusion shape mismatch"
    );
    let (m, k, n) = (a.rows(), a.cols(), b.cols());
    let mut tiles = BTreeMap::new();
    for (id, rows, cols, matrix_operand) in [
        (a.out(), m, k, true),
        (b.out(), k, n, true),
        (c.out(), 1, n, false),
        (d.out(), m, n, false),
        (e.out(), m, n, false),
        (r.out(), m, n, false),
    ] {
        tiles.insert(
            id,
            Tile {
                rows,
                cols,
                uses: 1,
                tile_uses: if matrix_operand { 8 } else { 1 },
                layout: "row_major_q1_31",
                residency: if matrix_operand {
                    "simdgroup_register"
                } else {
                    "register"
                },
            },
        );
    }
    let plan = Plan {
        kind: KernelKind::GemmBiasRelu,
        m,
        k,
        n,
        inputs: vec![a.name().into(), b.name().into(), c.name().into()],
        input_values: vec![a.out(), b.out(), c.out()],
        output: store.name().into(),
        tiles,
    };
    plan.validate()?;
    Ok(plan)
}

fn elementwise_plan(
    names: [&str; 3],
    ids: [u32; 4],
    shape: (u32, u32),
    bias: (u32, u32),
) -> Result<Plan> {
    let (m, n) = shape;
    ensure!(bias == (1, n), "row bias shape mismatch");
    ensure!(
        ids.iter().collect::<BTreeSet<_>>().len() == 4,
        "SSA redefinition"
    );
    let tiles = ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            (
                *id,
                Tile {
                    rows: if i == 1 { 1 } else { m },
                    cols: n,
                    uses: 1,
                    tile_uses: 1,
                    layout: "row_major_q1_31",
                    residency: "register",
                },
            )
        })
        .collect();
    let plan = Plan {
        kind: KernelKind::AddRelu,
        m,
        k: 1,
        n,
        inputs: vec![names[0].into(), names[1].into()],
        input_values: vec![ids[0], ids[1]],
        output: names[2].into(),
        tiles,
    };
    plan.validate()?;
    Ok(plan)
}
fn lower_elementwise(ops: &[Op]) -> Result<Plan> {
    let [Op::Load {
        name: a,
        rows: m,
        cols: n,
        out: ai,
    }, Op::Load {
        name: b,
        rows: br,
        cols: bc,
        out: bi,
    }, Op::Add {
        a: aa,
        b: ab,
        out: ad,
    }, Op::Relu { a: ra, out: r }, Op::Store { a: sa, name: c }] = ops
    else {
        bail!("unsupported elementwise graph")
    };
    ensure!(
        (aa, ab, ra, sa) == (ai, bi, ad, r),
        "unsupported elementwise edges"
    );
    elementwise_plan([a, b, c], [*ai, *bi, *ad, *r], (*m, *n), (*br, *bc))
}
fn lower_elementwise_view(trace: crate::trace_generated::wire::Trace<'_>) -> Result<Plan> {
    let ops = trace.ops();
    let a = ops
        .get(0)
        .operation_as_load()
        .ok_or_else(|| anyhow::anyhow!("expected load"))?;
    let b = ops
        .get(1)
        .operation_as_load()
        .ok_or_else(|| anyhow::anyhow!("expected bias load"))?;
    let add = ops
        .get(2)
        .operation_as_add()
        .ok_or_else(|| anyhow::anyhow!("expected add"))?;
    let relu = ops
        .get(3)
        .operation_as_relu()
        .ok_or_else(|| anyhow::anyhow!("expected relu"))?;
    let store = ops
        .get(4)
        .operation_as_store()
        .ok_or_else(|| anyhow::anyhow!("expected store"))?;
    ensure!(
        (add.a(), add.b(), relu.a(), store.a()) == (a.out(), b.out(), add.out(), relu.out()),
        "unsupported elementwise edges"
    );
    elementwise_plan(
        [a.name(), b.name(), store.name()],
        [a.out(), b.out(), add.out(), relu.out()],
        (a.rows(), a.cols()),
        (b.rows(), b.cols()),
    )
}

fn cordic_plan(
    function: u8,
    loads: Vec<(String, u32, u32, u32)>,
    a: u32,
    b: u32,
    out: u32,
    store_a: u32,
    output: String,
) -> Result<Plan> {
    let kind = match function {
        0 => KernelKind::Sin,
        1 => KernelKind::Cos,
        2 => KernelKind::Atan2,
        3 => KernelKind::Tanh,
        _ => bail!("unsupported CORDIC function"),
    };
    let count = if kind == KernelKind::Atan2 { 2 } else { 1 };
    ensure!(loads.len() == count, "CORDIC input count");
    let (_, m, n, first) = &loads[0];
    ensure!(a == *first && store_a == out, "CORDIC SSA edges");
    ensure!(
        b == if count == 2 { loads[1].3 } else { *first },
        "CORDIC second operand"
    );
    let ids = loads
        .iter()
        .map(|v| v.3)
        .chain([out])
        .collect::<BTreeSet<_>>();
    ensure!(ids.len() == count + 1, "CORDIC SSA redefinition");
    ensure!(
        loads.iter().all(|v| (v.1, v.2) == (*m, *n)),
        "CORDIC shape mismatch"
    );
    let tiles = loads
        .iter()
        .map(|v| {
            (
                v.3,
                Tile {
                    rows: v.1,
                    cols: v.2,
                    uses: 1,
                    tile_uses: 1,
                    layout: "row_major_q1_31",
                    residency: "register",
                },
            )
        })
        .collect();
    let plan = Plan {
        kind,
        m: *m,
        k: 1,
        n: *n,
        inputs: loads.iter().map(|v| v.0.clone()).collect(),
        input_values: loads.iter().map(|v| v.3).collect(),
        output,
        tiles,
    };
    plan.validate()?;
    Ok(plan)
}
fn lower_cordic(ops: &[Op]) -> Result<Plan> {
    ensure!(matches!(ops.len(), 3 | 4), "CORDIC graph length");
    let count = ops.len() - 2;
    let mut loads = Vec::new();
    for op in &ops[..count] {
        if let Op::Load {
            name,
            rows,
            cols,
            out,
        } = op
        {
            loads.push((name.clone(), *rows, *cols, *out));
        } else {
            bail!("expected CORDIC load")
        }
    }
    let Op::Cordic {
        function,
        a,
        b,
        out,
    } = ops[count]
    else {
        bail!("expected CORDIC operation")
    };
    let Op::Store {
        a: store_a,
        name: ref output,
    } = ops[count + 1]
    else {
        bail!("expected CORDIC store")
    };
    cordic_plan(function, loads, a, b, out, store_a, output.clone())
}
fn lower_cordic_view(trace: crate::trace_generated::wire::Trace<'_>) -> Result<Plan> {
    let ops = trace.ops();
    let count = ops.len() - 2;
    let mut loads = Vec::new();
    for i in 0..count {
        let op = ops
            .get(i)
            .operation_as_load()
            .ok_or_else(|| anyhow::anyhow!("expected CORDIC load"))?;
        loads.push((op.name().into(), op.rows(), op.cols(), op.out()));
    }
    let op = ops
        .get(count)
        .operation_as_cordic()
        .ok_or_else(|| anyhow::anyhow!("expected CORDIC operation"))?;
    let store = ops
        .get(count + 1)
        .operation_as_store()
        .ok_or_else(|| anyhow::anyhow!("expected CORDIC store"))?;
    cordic_plan(
        op.function(),
        loads,
        op.a(),
        op.b(),
        op.out(),
        store.a(),
        store.name().into(),
    )
}
