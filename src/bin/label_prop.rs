extern crate COST;

use std::fs::File;

use COST::graph_iterator::{EdgeMapper, DeltaCompressedReaderMapper, NodesEdgesMemMapper, UpperLowerMemMapper };
use std::io::BufReader;

fn main() {

    if std::env::args().len() != 4 {
        println!("Usage: label_propagation  (vertex | hilbert | compressed) <prefix> nodes");
        return;
    }

    let mode = std::env::args().nth(1).expect("mode unavailable");
    let name = std::env::args().nth(2).expect("name unavailable");
    let nodes: u32 = std::env::args().nth(3).expect("nodes unavailable").parse().expect("nodes not parseable");

    match mode.as_str() {
        "vertex" => {
            label_propagation(&NodesEdgesMemMapper::new(&name), nodes)
        },
        "hilbert" => {
            label_propagation(&UpperLowerMemMapper::new(&name), nodes)
        },
        "compressed" => {
            label_propagation(&DeltaCompressedReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())), nodes)
        },
        _ => { println!("unrecognized mode: {:?}", mode); },
    }
}

fn label_propagation<G: EdgeMapper>(graph: &G, nodes: u32) {

    let mut label: Vec<u32> = (0..nodes).collect();
    let mut old_sum: u64 = label.iter().fold(0, |t,x| t + *x as u64) + 1;
    let mut new_sum: u64 = label.iter().fold(0, |t,x| t + *x as u64);

    while new_sum < old_sum {
        graph.map_edges(|src, dst| {
            match label[src as usize].cmp(&label[dst as usize]) {
                std::cmp::Ordering::Less    => label[dst as usize] = label[src as usize],
                std::cmp::Ordering::Greater => label[src as usize] = label[dst as usize],
                std::cmp::Ordering::Equal   => { },
            }
        });

        old_sum = new_sum;
        new_sum = label.iter().fold(0, |t,x| t + *x as u64);
        println!("iteration");
    }

    let mut non_roots = 0u32;
    for i in 0..label.len() { if i as u32 != label[i] { non_roots += 1; }}
    println!("{} non-roots found", non_roots);
}