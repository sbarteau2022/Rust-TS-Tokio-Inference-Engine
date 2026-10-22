//! Linear GPU stage composition: first input of stage i>0 consumes stage i-1 output.
use crate::{
    execute::{self, Inputs},
    ir::Plan,
};
use anyhow::{ensure, Result};

pub(crate) fn validate(plans: &[Plan], inputs: &Inputs) -> Result<()> {
    ensure!((1..=16).contains(&plans.len()), "chain stage bound 1..16");
    for (stage, plan) in plans.iter().enumerate() {
        plan.validate()?;
        for (slot, (name, len)) in plan.inputs.iter().zip(plan.input_lengths()).enumerate() {
            if stage > 0 && slot == 0 {
                let prior = &plans[stage - 1];
                let tile = &plan.tiles[&plan.input_values[0]];
                ensure!(
                    tile.rows == prior.m && tile.cols == prior.n,
                    "chain activation shape mismatch"
                );
            } else {
                ensure!(
                    inputs.get(name).is_some_and(|v| v.len() == len),
                    "chain input binding/length: {name}"
                );
            }
        }
    }
    Ok(())
}
pub(crate) fn reference(plans: &[Plan], inputs: &Inputs) -> Result<Vec<i32>> {
    validate(plans, inputs)?;
    let mut previous = None;
    for plan in plans {
        let mut bound = Inputs::new();
        for (slot, name) in plan.inputs.iter().enumerate() {
            let value = if slot == 0 && previous.is_some() {
                previous.take().unwrap()
            } else {
                inputs[name].clone()
            };
            bound.insert(name.clone(), value);
        }
        previous = Some(execute::reference(plan, &bound)?);
    }
    Ok(previous.expect("validated nonempty chain"))
}
pub(crate) struct ValidationInput {
    pub plans: Vec<Plan>,
    pub inputs: Inputs,
}
