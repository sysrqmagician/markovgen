/* markovgen
Copyright 2024 sysrqmagician <sysrqmagician@proton.me>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

use smartstring::alias::String;
use std::{collections::HashMap, error::Error, fmt::Display, sync::Arc};

use rand::Rng;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Vertex {
    value: char,
    edges: Vec<Edge>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
struct Edge {
    vertex_index: usize,
    probability: f32,
}

struct ProtoVertex {
    value: char,
    char_ref_counts: HashMap<char, usize>,
}

pub struct GraphConstructor {
    vertices: Vec<ProtoVertex>,
}

impl From<GraphConstructor> for Graph {
    fn from(constructor: GraphConstructor) -> Self {
        let mut constructed_graph = Graph {
            vertices: Vec::new(),
        };

        let mut discovered_dead_ends: Vec<Vertex> = Vec::new();

        for (proto_index, proto) in constructor.vertices.iter().enumerate() {
            let mut vertex = Vertex {
                value: proto.value,
                edges: Vec::new(),
            };

            let ref_counts_sum: usize = proto.char_ref_counts.values().sum();
            for edge_ref in proto.char_ref_counts.iter() {
                let mut ref_index: Option<usize> = None;
                for (other_proto_index, other_proto) in constructor.vertices.iter().enumerate() {
                    if other_proto.value == *edge_ref.0 {
                        ref_index = Some(other_proto_index);
                        break;
                    }
                }

                if ref_index.is_none() {
                    for (index, dead_end) in discovered_dead_ends.iter().enumerate() {
                        if dead_end.value == *edge_ref.0 {
                            ref_index = Some(constructor.vertices.len() + index);
                        }
                    }
                }

                if ref_index.is_none() {
                    // println!(
                    //     "Unable to find ref_index for value '{}' (0x{:x}). Creating new vertex.",
                    //     *edge_ref.0, *edge_ref.0 as i32,
                    // );
                    let new_vertex = Vertex {
                        value: *edge_ref.0,
                        edges: Vec::new(),
                    };
                    discovered_dead_ends.push(new_vertex);

                    ref_index = Some(constructor.vertices.len() + discovered_dead_ends.len() - 1);
                }

                let ref_index = ref_index.unwrap();
                vertex.edges.push(Edge {
                    vertex_index: ref_index,
                    probability: *edge_ref.1 as f32 / ref_counts_sum as f32,
                });
            }

            vertex
                .edges
                .sort_by(|a, b| a.probability.partial_cmp(&b.probability).unwrap());
            constructed_graph.vertices.insert(proto_index, vertex);
        }
        for dead_end_vertex in discovered_dead_ends {
            constructed_graph.vertices.push(dead_end_vertex);
        }

        constructed_graph
    }
}

impl GraphConstructor {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
        }
    }

    pub fn register_sequence(&mut self, current: char, next: char) {
        for vertex in self.vertices.iter_mut() {
            if vertex.value == current {
                match vertex.char_ref_counts.get_mut(&next) {
                    Some(c) => {
                        *c += 1;
                    }
                    None => {
                        vertex.char_ref_counts.insert(next, 1);
                    }
                }

                return;
            }
        }

        let mut new_vertex = ProtoVertex {
            value: current,
            char_ref_counts: HashMap::new(),
        };
        new_vertex.char_ref_counts.insert(next, 1);

        self.vertices.push(new_vertex);
    }

    pub fn construct(self) -> Graph {
        self.into()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Graph {
    vertices: Vec<Vertex>,
}

#[derive(Clone)]
pub struct GraphStepper {
    graph: Arc<Graph>,
    position: usize,
    built_string: String,
    configuration: GraphStepperConfiguration,
}

#[derive(Clone)]
#[allow(non_snake_case)]
pub struct GraphStepperConfiguration {
    pub start_char: Option<char>,
    pub min_length: Option<usize>,
}

#[derive(Debug)]
pub enum GraphStepperError {
    InvalidParameter(InvalidConfigurationParameter),
    EdgeExhaustion,
}

impl Display for GraphStepperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_message: std::string::String = match self {
            GraphStepperError::EdgeExhaustion => {
                "No edges left to create configuration-compliant sample".to_string()
            }
            GraphStepperError::InvalidParameter(param) => {
                let param_name = match param {
                    InvalidConfigurationParameter::EndChar => "End Char",
                    InvalidConfigurationParameter::StartChar => "Start Char",
                };
                format!("Invalid parameter provided: {param_name}")
            }
        };

        f.write_str(&error_message)
    }
}

impl Error for GraphStepperError {}

#[derive(Debug)]
pub enum InvalidConfigurationParameter {
    StartChar,
    EndChar,
}

impl GraphStepper {
    pub fn new(
        graph: Arc<Graph>,
        configuration: GraphStepperConfiguration,
    ) -> Result<Self, GraphStepperError> {
        let mut out = Self {
            graph,
            position: 0,
            built_string: String::new(),
            configuration,
        };

        if out.configuration.start_char.is_some()
            && out
                .find_position(out.configuration.start_char.unwrap())
                .is_none()
        {
            return Err(GraphStepperError::InvalidParameter(
                InvalidConfigurationParameter::StartChar,
            ));
        }

        out.reset_position();
        Ok(out)
    }

    fn reset_position(&mut self) {
        match self.configuration.start_char {
            Some(x) => self.position = self.find_position(x).unwrap(), // Checked in constructor
            None => self.position = self.random_position(),
        }
    }

    fn find_position(&self, value: char) -> Option<usize> {
        for (index, vertex) in self.graph.vertices.iter().enumerate() {
            if vertex.value == value {
                return Some(index);
            }
        }

        None
    }

    pub fn step(&mut self) -> Result<(), GraphStepperError> {
        let random_value: f32 = rand::thread_rng().gen_range(0.0..1.0);
        let mut selection: Option<usize> = None;

        let mut edges = &self.get_current_vertex().edges;

        let mut override_edges: Vec<Edge>;
        if let Some(min_length) = self.configuration.min_length {
            if self.built_string.chars().count() < min_length {
                override_edges = Vec::with_capacity(edges.capacity());

                let mut lost_prob = 0.0;
                for edge in edges.iter() {
                    if self
                        .graph
                        .vertices
                        .get(edge.vertex_index)
                        .unwrap()
                        .edges
                        .is_empty()
                    {
                        lost_prob += edge.probability;
                    } else {
                        override_edges.push(edge.clone());
                    }
                }
                lost_prob /= override_edges.len() as f32;

                for edge in override_edges.iter_mut() {
                    edge.probability += lost_prob; // Could floating point addition error bring the
                                                   // edge array out of order?
                }

                edges = &override_edges;
            }
        }

        // The edges are ordered by probability, so we can just go from the top
        // Here, the "Roulette Wheel Selection" algorithm is implemented.
        let mut prob_sum = 0.0;
        for edge in edges {
            prob_sum += edge.probability;
            if random_value < prob_sum {
                selection = Some(edge.vertex_index);
                break;
            }
        }

        if selection.is_none() {
            // Use last; this is an edge-case that might be reached by floating-pointer addition error.
            // Probabilities might not add up to be high enough, but in any case this would then
            // select the last, most probable value.

            let last = self.get_current_vertex().edges.last();
            match last {
                Some(x) => selection = Some(x.vertex_index),
                None => {
                    // Reached dead end
                    return Err(GraphStepperError::EdgeExhaustion);
                }
            }
        }

        let new_position = selection.unwrap();
        self.position = new_position;
        self.built_string.push(self.get_current_vertex().value);

        Ok(())
    }

    pub fn step_until(
        &mut self,
        value: char,
        timeout: usize,
    ) -> Result<GraphStepperOut, GraphStepperError> {
        if self.find_position(value).is_none() {
            return Err(GraphStepperError::InvalidParameter(
                InvalidConfigurationParameter::EndChar,
            ));
        }

        loop {
            if self.built_string.ends_with(value) {
                return Ok(GraphStepperOut::Reached(self.flush()));
            }

            if self.built_string.len() >= timeout {
                return Ok(GraphStepperOut::Timeout(self.flush()));
            }

            if let Err(_exhaustion) = self.step() {
                return Ok(GraphStepperOut::Exhausted(self.flush()));
            }
        }
    }

    pub fn step_until_end_state(
        &mut self,
        timeout: usize,
    ) -> Result<GraphStepperOut, GraphStepperError> {
        loop {
            if self.built_string.len() >= timeout {
                return Ok(GraphStepperOut::Timeout(self.flush()));
            }

            if let Err(_exhaustion) = self.step() {
                return Ok(GraphStepperOut::Reached(self.flush()));
            }
        }
    }

    pub fn flush(&mut self) -> String {
        let out = self.built_string.clone();
        self.built_string = String::new();

        self.reset_position();

        out
    }

    fn random_position(&self) -> usize {
        rand::thread_rng().gen_range(0..self.graph.vertices.len())
    }

    fn get_current_vertex(&self) -> &Vertex {
        self.graph.vertices.get(self.position).unwrap()
    }
}

pub enum GraphStepperOut {
    Reached(String),
    Timeout(String),
    /// Not conforming to configuration, but out of edges pointing to anything other than end states.
    Exhausted(String),
}

impl Display for GraphStepperOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            GraphStepperOut::Timeout(x) => x,
            GraphStepperOut::Reached(x) => x,
            GraphStepperOut::Exhausted(x) => x,
        };

        f.write_fmt(format_args!("{}", out))
    }
}
