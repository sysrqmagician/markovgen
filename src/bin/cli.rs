use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter},
    path::PathBuf,
    sync::Arc,
};

use clap::{CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::Shell;
use markovgen::*;

#[derive(Parser)]
#[command(name = "markovcli", version, about)]
struct CliArgs {
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    #[command(about = "Build a markov chain graph from a dataset of sequences.")]
    Compile {
        #[arg(help = "Path to an input file with sequences separated by newlines.", value_hint = ValueHint::FilePath)]
        input_path: PathBuf,
        #[arg(help = "Defaults to [input_name].graph.bin", value_hint = ValueHint::FilePath)]
        output_path: Option<PathBuf>,
    },
    #[command(about = "Sample a sequence from a previously compiled graph.")]
    Sample {
        #[arg(help = "Path to a previously compiled graph.", value_hint = ValueHint::FilePath)]
        graph_path: PathBuf,
        #[arg(help = "The amount of sequences to sample.", default_value = "1")]
        count: usize,
        #[arg(
            long,
            help = "Minimum length of sample readable characters.",
            default_value = "3"
        )]
        min_length: usize,
        #[arg(
            long,
            help = "Maximum number of bytes generated per sequence",
            default_value = "64"
        )]
        max_bytes: usize,
    },
    #[command(about = "Generate shell completion script to STDOUT.")]
    GenerateCompletions {
        #[arg(value_enum)]
        shell_generator: Shell,
    },
}

const SEQUENCE_START: char = '\x01';
const SEQUENCE_END: char = '\x02';

fn main() {
    let args = CliArgs::parse();

    match args.command {
        Subcommands::GenerateCompletions { shell_generator } => {
            let mut command = CliArgs::command();
            let command = &mut command;

            clap_complete::generate(
                shell_generator,
                command,
                command.get_name().to_string(),
                &mut io::stdout(),
            );
        }

        Subcommands::Compile {
            input_path: input,
            output_path: output,
        } => {
            let input_file = OpenOptions::new().read(true).open(&input);
            if input_file.is_err() {
                println!("Unable to open input file: {}", input_file.unwrap_err());
                return;
            }
            let input_file = input_file.unwrap();
            let input_reader = BufReader::new(input_file);

            let mut constructor = GraphConstructor::new();
            for line in input_reader.lines() {
                if line.is_err() {
                    println!("Unable to read line: {}", line.unwrap_err());
                    return;
                }

                let line = line.unwrap();
                let mut last_char = SEQUENCE_START;

                for char in line.chars() {
                    constructor.register_sequence(last_char, char);
                    last_char = char;
                }

                constructor.register_sequence(last_char, SEQUENCE_END);
            }

            println!("Constructed graph. Now computing probabilities.");
            let graph = constructor.construct();

            println!("Done. Writing file.");
            let output_path: PathBuf = match output {
                None => {
                    let mut out = input.clone();
                    out.set_extension("graph.bin");

                    out
                }
                Some(x) => x,
            };
            let output_file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output_path);
            if output_file.is_err() {
                println!("Unable to open output file: {}", output_file.unwrap_err());
                return;
            }
            let output_writer = BufWriter::new(output_file.unwrap());

            let serialization_result = bincode::serialize_into(output_writer, &graph);
            if serialization_result.is_err() {
                println!("Error while writing: {}", serialization_result.unwrap_err());
            }
        }

        Subcommands::Sample {
            graph_path,
            count,
            min_length: min_length_input,
            max_bytes,
        } => {
            if count == 0 {
                println!("Sample count can't be 0.");
                return;
            }

            let input_file = OpenOptions::new().read(true).open(graph_path);
            if input_file.is_err() {
                println!("Unable to open graph file: {}", input_file.unwrap_err());
                return;
            }
            let input_file = input_file.unwrap();
            let input_reader = BufReader::new(input_file);

            let graph = bincode::deserialize_from::<BufReader<File>, Graph>(input_reader);
            if graph.is_err() {
                println!("Unable to parse graph file.");
                return;
            }

            let graph = graph.unwrap();

            let min_length: Option<usize> = if min_length_input == 0 {
                None
            } else {
                Some(min_length_input)
            };

            let mut stepper = match GraphStepper::new(
                Arc::new(graph),
                GraphStepperConfiguration {
                    start_char: Some(SEQUENCE_START),
                    min_length,
                },
            ) {
                Ok(x) => x,
                Err(x) => {
                    match x {
                        GraphStepperError::InvalidParameter(
                            InvalidConfigurationParameter::StartChar,
                        ) => {
                            println!(
                                "Invalid graph file provided. SEQUENCE_START char is not present."
                            );
                        }
                        GraphStepperError::InvalidParameter(
                            InvalidConfigurationParameter::EndChar,
                        ) => {
                            println!(
                                "Invalid graph file provided. SEQUENCE_END char is not present."
                            );
                        }
                        _ => {
                            println!("An unexpected error occurred during sampling.")
                        }
                    }
                    return;
                }
            };
            let mut iteration = 0;
            while iteration < count {
                match stepper.step_until_end_state(max_bytes) {
                    Ok(GraphStepperOut::Reached(out)) => {
                        #[cfg(not(feature = "cli_no_print"))]
                        println!("{out}");
                        iteration += 1;
                    }
                    Ok(GraphStepperOut::Timeout(_)) | Ok(GraphStepperOut::Exhausted(_)) => {}
                    Err(error) => {
                        println!("Error during sampling: {error}");
                    }
                }
            }
        }
    }
}
