use crate::trace_generated::wire;
use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Op {
    Load {
        name: String,
        rows: u32,
        cols: u32,
        out: u32,
    },
    Dot {
        a: u32,
        b: u32,
        out: u32,
    },
    Add {
        a: u32,
        b: u32,
        out: u32,
    },
    Relu {
        a: u32,
        out: u32,
    },
    Cordic {
        function: u8,
        a: u32,
        b: u32,
        out: u32,
    },
    Store {
        a: u32,
        name: String,
    },
}
pub fn verified(bytes: &[u8]) -> Result<wire::Trace<'_>> {
    ensure!(
        bytes.len() >= 8 && bytes.len() <= 65536,
        "trace size outside 8..65536"
    );
    ensure!(
        wire::trace_buffer_has_identifier(bytes),
        "invalid trace identifier"
    );
    let trace = wire::root_as_trace(bytes)?;
    ensure!(trace.version() == 3, "unsupported version");
    let ops = trace.ops();
    ensure!(!ops.is_empty() && ops.len() <= 64, "operation budget");
    Ok(trace)
}
/// Debug/round-trip projection only. Runtime lowering uses borrowed verified views.
pub fn decode(bytes: &[u8]) -> Result<Vec<Op>> {
    let trace = verified(bytes)?;
    let ops = trace.ops();
    ops.iter()
        .map(|op| {
            Ok(match op.operation_type() {
                wire::Operation::Load => {
                    let x = op
                        .operation_as_load()
                        .ok_or_else(|| anyhow::anyhow!("missing load"))?;
                    Op::Load {
                        name: x.name().into(),
                        rows: x.rows(),
                        cols: x.cols(),
                        out: x.out(),
                    }
                }
                wire::Operation::Dot => {
                    let x = op
                        .operation_as_dot()
                        .ok_or_else(|| anyhow::anyhow!("missing dot"))?;
                    Op::Dot {
                        a: x.a(),
                        b: x.b(),
                        out: x.out(),
                    }
                }
                wire::Operation::Add => {
                    let x = op
                        .operation_as_add()
                        .ok_or_else(|| anyhow::anyhow!("missing add"))?;
                    Op::Add {
                        a: x.a(),
                        b: x.b(),
                        out: x.out(),
                    }
                }
                wire::Operation::Relu => {
                    let x = op
                        .operation_as_relu()
                        .ok_or_else(|| anyhow::anyhow!("missing relu"))?;
                    Op::Relu {
                        a: x.a(),
                        out: x.out(),
                    }
                }
                wire::Operation::Cordic => {
                    let x = op
                        .operation_as_cordic()
                        .ok_or_else(|| anyhow::anyhow!("missing CORDIC"))?;
                    Op::Cordic {
                        function: x.function(),
                        a: x.a(),
                        b: x.b(),
                        out: x.out(),
                    }
                }
                wire::Operation::Store => {
                    let x = op
                        .operation_as_store()
                        .ok_or_else(|| anyhow::anyhow!("missing store"))?;
                    Op::Store {
                        a: x.a(),
                        name: x.name().into(),
                    }
                }
                _ => bail!("unsupported operation"),
            })
        })
        .collect()
}
