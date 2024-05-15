use criterion::{criterion_group, criterion_main, Criterion};
use markovgen::*;
use std::sync::Arc;

const NAME_DATASET: &str = include_str!("US_Census_1990_Frequent_Male_First_Names.txt");

const SEQUENCE_START: char = '\x01';

const GRAPH_STEPPER_CONFIG: GraphStepperConfiguration = GraphStepperConfiguration {
    start_char: Some(SEQUENCE_START),
    min_length: None,
};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("construct graph", |b| {
        b.iter(|| {
            let mut constructor = GraphConstructor::new();
            NAME_DATASET.lines().for_each(|l| {
                l.chars().fold(SEQUENCE_START, |acc, x| {
                    constructor.register_sequence(acc, x);
                    x
                });
            });
            constructor.construct();
        })
    });

    let mut constructor = GraphConstructor::new();
    NAME_DATASET.lines().for_each(|l| {
        l.chars().fold(SEQUENCE_START, |acc, x| {
            constructor.register_sequence(acc, x);
            x
        });
    });
    let graph = Arc::new(constructor.construct());

    c.bench_function("build stepper", |b| {
        b.iter(|| {
            let _ = GraphStepper::new(graph.clone(), GRAPH_STEPPER_CONFIG)
                .expect("Unable to build stepper");
        });
    });

    let mut stepper =
        GraphStepper::new(graph.clone(), GRAPH_STEPPER_CONFIG).expect("Unable to build stepper");
    c.bench_function(
        "pre-built stepper stepping once and resetting (no min length)",
        |b| {
            b.iter(|| {
                let _ = stepper.step();
                let _ = stepper.flush();
            });
        },
    );
    drop(stepper);

    let mut stepper = GraphStepper::new(
        graph.clone(),
        GraphStepperConfiguration {
            min_length: Some(1),
            ..GRAPH_STEPPER_CONFIG
        },
    )
    .expect("Unable to build stepper");
    c.bench_function(
        "pre-built stepper stepping once and resetting (min length 1)",
        |b| {
            b.iter(|| {
                let _ = stepper.step();
                let _ = stepper.flush();
            });
        },
    );
    drop(stepper);
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
