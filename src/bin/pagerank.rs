extern crate COST;

use std::fs::File;

use COST::graph_iterator::{EdgeMapper, DeltaCompressedReaderMapper, NodesEdgesMemMapper, UpperLowerMemMapper };
use std::io::BufReader;

fn main() {

    if std::env::args().len() != 4 {
        println!("Usage: pagerank  (vertex | hilbert | compressed) <prefix> nodes");
        return;
    }

    let mode = std::env::args().nth(1).expect("mode unavailable");
    let name = std::env::args().nth(2).expect("name unavailable");
    let nodes: u32 = std::env::args().nth(3).expect("nodes unavailable").parse().expect("nodes not parseable");

    match mode.as_str() {
        "vertex" => {
            pagerank(&NodesEdgesMemMapper::new(&name), nodes, 0.85f32)
        },
        "hilbert" => {
            pagerank(&UpperLowerMemMapper::new(&name), nodes, 0.85f32)
        },
        "compressed" => {
            pagerank(&DeltaCompressedReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())), nodes, 0.85f32)
        },
        _ => { println!("unrecognized mode: {:?}", mode); },
    }
}

fn pagerank<G: EdgeMapper>(graph: &G, nodes: u32, alpha: f32) {

    let timer = std::time::Instant::now();

    let mut src = vec![0f32; nodes as usize];
    let mut dst = vec![0f32; nodes as usize];
    let mut deg = vec![0f32; nodes as usize];

    graph.map_edges(|x, _| { deg[x as usize] += 1f32 });

    for _iteration in 0 .. 20 {
        println!("Iteration {}:\t{:?}", _iteration, timer.elapsed());
        for node in 0 .. nodes {
            src[node as usize] = alpha * dst[node as usize] / deg[node as usize];
            dst[node as usize] = 1f32 - alpha;
        }

        // graph.map_edges(|x, y| { dst[y as usize] += src[x as usize]; });

        // UNSAFE:
        graph.map_edges(|x, y| { unsafe { *dst.get_unchecked_mut(y as usize) += *src.get_unchecked(x as usize); }});
    }
}