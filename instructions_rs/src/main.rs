use std::collections::HashMap;

mod action;
mod output;

mod instructions;
mod opcode;
mod step_template;

use action::Action;
use action::action_to_output;
use output::Output;
use std::sync::LazyLock;

fn step_template_to_output(step_istr: &step_template::StepTemplate) -> Output {
    let mut result = Output::new();

    let actions: Vec<_> = step_istr.iter().collect();
    let outputs: Vec<_> = step_istr.iter().map(action_to_output).collect();

    // loop through all unique pairs
    for i in 1..outputs.len() - 1 {
        for j in i + 1..outputs.len() {
            if outputs[i].intersect(&outputs[j]) {
                eprintln!(
                    "Failed when merging actions {:?} and {:?}",
                    *actions[i], *actions[j]
                );

                // TODO: print error?
                return result;
            }
        }
    }

    for output in &outputs {
        result.merge(output);
    }

    result
}

fn main() {
    println!("Hello, world!");
    instructions::build_all_instructions();
}
