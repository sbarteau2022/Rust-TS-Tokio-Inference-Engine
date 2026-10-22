//! Validated native attention, recurrence and expert-routing tensor operators.
use super::*;
fn descriptor(shape: Vec<i32>, dtype: i32) -> Descriptor {
    let elements = shape
        .iter()
        .fold(1usize, |a, v| a.saturating_mul(*v as usize));
    Descriptor {
        shape,
        dtype,
        elements,
    }
}
impl Graph {
    fn kernel(
        &mut self,
        code: i32,
        operands: Vec<NodeId>,
        ints: Vec<i32>,
        floats: Vec<f32>,
        outputs: Vec<Descriptor>,
        extra: usize,
    ) -> Result<Vec<NodeId>> {
        ensure!(
            self.nodes.len() + outputs.len() <= MAX_NODES,
            "kernel node cap"
        );
        let bytes = outputs
            .iter()
            .try_fold(extra, |a, d| {
                d.elements.checked_mul(4).and_then(|b| a.checked_add(b))
            })
            .context("kernel byte overflow")?;
        let total = self
            .graph_bytes
            .checked_add(bytes)
            .filter(|n| *n <= MAX_GRAPH_BYTES)
            .context("kernel graph byte cap")?;
        let first = self.nodes.len() as u32;
        self.nodes.push(Node::Kernel {
            code,
            operands,
            ints,
            floats,
        });
        for _ in 1..outputs.len() {
            self.nodes.push(Node::KernelOutput);
        }
        let ids = (first..first + outputs.len() as u32).collect();
        self.descriptors.extend(outputs);
        self.graph_bytes = total;
        Ok(ids)
    }
    pub fn rope(&mut self, x: NodeId, dims: i32, offset: i32, base: f32) -> Result<NodeId> {
        let a = self.descriptor(x)?;
        ensure!(
            a.dtype == 0
                && a.shape.len() == 4
                && dims > 0
                && dims % 2 == 0
                && dims <= a.shape[3]
                && (0..=262144).contains(&offset)
                && base.is_finite()
                && base > 1.0,
            "invalid RoPE descriptor"
        );
        let output = a.clone();
        Ok(self.kernel(1, vec![x], vec![dims, offset], vec![base], vec![output], 0)?[0])
    }
    pub fn attention(
        &mut self,
        q: NodeId,
        k: NodeId,
        v: NodeId,
        scale: f32,
        causal: bool,
    ) -> Result<NodeId> {
        let qd = self.descriptor(q)?;
        let kd = self.descriptor(k)?;
        let vd = self.descriptor(v)?;
        ensure!(
            qd.dtype == 0
                && kd.dtype == 0
                && vd.dtype == 0
                && qd.shape.len() == 4
                && kd.shape.len() == 4
                && vd.shape.len() == 4,
            "attention dtype/rank"
        );
        ensure!(
            qd.shape[0] == kd.shape[0]
                && kd.shape[0] == vd.shape[0]
                && kd.shape[1] == vd.shape[1]
                && qd.shape[1] % kd.shape[1] == 0
                && kd.shape[2] == vd.shape[2]
                && qd.shape[3] == kd.shape[3]
                && qd.shape[2] <= kd.shape[2]
                && scale.is_finite()
                && scale > 0.0,
            "attention shape/scale mismatch"
        );
        let out = descriptor(vec![qd.shape[0], qd.shape[1], qd.shape[2], vd.shape[3]], 0);
        // Conservative fallback attention-score workspace, even if the fused path avoids it.
        let extra = [qd.shape[0], qd.shape[1], qd.shape[2], kd.shape[2]]
            .iter()
            .try_fold(4usize, |a, b| a.checked_mul(*b as usize))
            .context("attention workspace overflow")?;
        Ok(self.kernel(
            2,
            vec![q, k, v],
            vec![i32::from(causal)],
            vec![scale],
            vec![out],
            extra,
        )?[0])
    }
    /// q/k are already normalized; decay and beta are already transformed.
    /// Returns output [B,T,Hv,Dv] and FP32 state [B,Hv,Dv,Dk].
    pub fn gated_delta(&mut self, inputs: [NodeId; 6]) -> Result<(NodeId, NodeId)> {
        let d = inputs
            .iter()
            .map(|i| self.descriptor(*i))
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            d.iter().all(|v| v.dtype == 0)
                && d[0].shape.len() == 4
                && d[0].shape == d[1].shape
                && d[2].shape.len() == 4,
            "gated-delta dtype/rank"
        );
        let (b, t, hk, dk) = (d[0].shape[0], d[0].shape[1], d[0].shape[2], d[0].shape[3]);
        let (hv, dv) = (d[2].shape[2], d[2].shape[3]);
        ensure!(
            t <= 128
                && d[2].shape[..2] == [b, t]
                && hv % hk == 0
                && d[3].shape == [b, t, hv]
                && d[4].shape == [b, t, hv]
                && d[5].shape == [b, hv, dv, dk],
            "gated-delta state/head mismatch"
        );
        let output = descriptor(vec![b, t, hv, dv], 0);
        let state = d[5].clone();
        let extra = state
            .elements
            .checked_mul(t as usize)
            .and_then(|v| v.checked_mul(16))
            .context("recurrent workspace overflow")?;
        let ids = self.kernel(
            3,
            inputs.to_vec(),
            vec![],
            vec![],
            vec![output, state],
            extra,
        )?;
        Ok((ids[0], ids[1]))
    }
    /// Same top-k/renormalization semantics as MLX-LM's sparse MoE router.
    pub fn route_topk(&mut self, logits: NodeId, top: i32) -> Result<(NodeId, NodeId)> {
        let d = self.descriptor(logits)?;
        ensure!(
            d.dtype == 0 && d.shape.len() == 2 && top > 0 && top <= d.shape[1],
            "invalid expert top-k"
        );
        let shape = vec![d.shape[0], top];
        let extra = d.elements * 12;
        let ids = self.kernel(
            4,
            vec![logits],
            vec![top],
            vec![],
            vec![descriptor(shape.clone(), 2), descriptor(shape, 0)],
            extra,
        )?;
        Ok((ids[0], ids[1]))
    }
    pub fn gather_quantized(
        &mut self,
        x: NodeId,
        packed: NodeId,
        scales: NodeId,
        biases: NodeId,
        indices: NodeId,
        quant: super::super::weights::Quantization,
    ) -> Result<NodeId> {
        ensure!(
            [2, 3, 4, 5, 6, 8].contains(&quant.bits) && [32, 64, 128].contains(&quant.group),
            "unsupported expert quantization"
        );
        let a = self.descriptor(x)?;
        let w = self.descriptor(packed)?;
        let s = self.descriptor(scales)?;
        let b = self.descriptor(biases)?;
        let idx = self.descriptor(indices)?;
        ensure!(
            a.dtype == 0
                && a.shape.len() == 2
                && w.dtype == 2
                && w.shape.len() == 3
                && [0, 3, 4].contains(&s.dtype)
                && s.dtype == b.dtype
                && s.shape.len() == 3
                && s.shape == b.shape
                && idx.dtype == 2
                && idx.shape.len() == 2
                && idx.shape[0] == a.shape[0],
            "expert dtype/rank mismatch"
        );
        ensure!(
            a.shape[1] % quant.group == 0
                && w.shape[2] as i64 == a.shape[1] as i64 * quant.bits as i64 / 32
                && s.shape == [w.shape[0], w.shape[1], a.shape[1] / quant.group],
            "expert packed shape mismatch"
        );
        let in_range = match &self.nodes[indices as usize] {
            Node::Input(Data::U32(values)) => values.iter().all(|i| *i < w.shape[0] as u32),
            Node::Kernel {
                code: 4, operands, ..
            } => self.descriptor(operands[0])?.shape[1] <= w.shape[0],
            _ => false,
        };
        ensure!(in_range, "untrusted or out-of-range expert indices");
        let output = descriptor(vec![a.shape[0], idx.shape[1], w.shape[1]], 0);
        Ok(self.kernel(
            5,
            vec![x, packed, scales, biases, indices],
            vec![quant.group, quant.bits],
            vec![],
            vec![output],
            0,
        )?[0])
    }
    pub fn mix_experts(&mut self, experts: NodeId, scores: NodeId) -> Result<NodeId> {
        let e = self.descriptor(experts)?;
        let s = self.descriptor(scores)?;
        ensure!(
            e.dtype == 0 && s.dtype == 0 && e.shape.len() == 3 && s.shape == e.shape[..2],
            "expert mix shape/dtype mismatch"
        );
        let out = descriptor(vec![e.shape[0], e.shape[2]], 0);
        let extra = e.elements * 4;
        Ok(self.kernel(6, vec![experts, scores], vec![], vec![], vec![out], extra)?[0])
    }
    pub fn silu(&mut self, x: NodeId) -> Result<NodeId> {
        let d = self.descriptor(x)?;
        ensure!(d.dtype == 0, "SiLU requires F32");
        let output = d.clone();
        Ok(self.kernel(7, vec![x], vec![], vec![], vec![output], 0)?[0])
    }
}
impl Graph {
    pub(crate) fn join_flat(&mut self, a: NodeId, b: NodeId) -> Result<NodeId> {
        let a_d = self.descriptor(a)?;
        let b_d = self.descriptor(b)?;
        ensure!(
            a_d.dtype == 0 && b_d.dtype == 0,
            "state output requires F32"
        );
        let count = a_d
            .elements
            .checked_add(b_d.elements)
            .and_then(|n| i32::try_from(n).ok())
            .context("joined output overflow")?;
        Ok(self.kernel(
            8,
            vec![a, b],
            vec![],
            vec![],
            vec![descriptor(vec![count], 0)],
            0,
        )?[0])
    }
}
