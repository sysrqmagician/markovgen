use std::sync::Arc;

use markovgen::*;

const NAME_DATASET: &str = "Tim\nTom\nThomas\nNathan\nNina\nTiara\nTyra\nTyrone";

const SEQUENCE_START: char = '\x01';
const SEQUENCE_END: char = '\x02';

fn main() {
    let mut constructor = GraphConstructor::new();
    NAME_DATASET.lines().for_each(|l| {
        l.chars().fold(SEQUENCE_START, |acc, x| {
            constructor.register_sequence(acc, x);
            x
        });

        constructor.register_sequence(l.chars().last().unwrap(), SEQUENCE_END);
    });
    let graph = Arc::new(constructor.construct());

    let mut stepper = GraphStepper::new(
        graph,
        GraphStepperConfiguration {
            start_char: Some(SEQUENCE_START),
            min_length: Some(3),
        },
    )
    .unwrap();

    // Step until reaching a "dead end" vertex, with a timeout of 16 steps.
    println!("{}", stepper.step_until_end_state(16).unwrap());
}
