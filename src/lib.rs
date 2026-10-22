pub mod execute;
pub mod gpu;
pub mod ir;
pub mod runtime;
pub mod telemetry;
#[allow(
    unused_imports,
    dead_code,
    unsafe_op_in_unsafe_fn,
    mismatched_lifetime_syntaxes,
    clippy::missing_safety_doc,
    clippy::extra_unused_lifetimes
)]
pub mod trace_generated;
pub mod wire;

pub mod backend;
pub mod cordic;
mod cordic_tables;

pub mod completion;

pub mod macro_bench;

mod chain;
pub mod sampled;

pub mod cache;

pub mod serving;
