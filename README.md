# markovgen
A library for building markov chain graphs from text datasets and performantly generating text sequences by traversing them.

## Features
- Simple API for building and traversing graphs
- Configurable minimum sequence length
- An example CLI application (markovcli) that supports building graphs from datasets and writing them to the disk, as well as sampling such graphs with customizable sequence length.
  - Capable of generating >2.5 million names per second with default settings (and cli_no_print feature set to avoid IO overhead) on my machine from the first_names benchmark dataset (see ``benches/``)
  - Try it using ``cargo run -r -F serde --bin markovcli``

## Example
``src/bin/example.rs``:
```rust
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
```

Running this should yield something like:
```
$ cargo run --bin example
Ninathom
```


## Plans for 1.0
- This was one of my first Rust projects, which I just cleaned up a little. I'll probably be changing the API to make it a little more ergonomic before the 1.0.0 release
- Proper multi-threading support (I think just cloning GraphSteppers and using them in different tasks should already work, but haven't actually tried it)
- Generic implementation to allow for String and char vertices (currently only chars are supported since this fit my original use-case of generating names)
