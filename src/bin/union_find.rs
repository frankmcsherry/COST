extern crate COST;

use std::fs::File;

use COST::graph_iterator::{EdgeMapper, DeltaCompressedReaderMapper, NodesEdgesMemMapper, UpperLowerMemMapper };
use std::io::BufReader;

fn main() {

    if std::env::args().len() != 4 {
        println!("Usage: union_find  (vertex | hilbert | compressed) <prefix> nodes");
        return;
    }

    let mode = std::env::args().nth(1).expect("mode unavailable");
    let name = std::env::args().nth(2).expect("name unavailable");
    let nodes: u32 = std::env::args().nth(3).expect("nodes unavailable").parse().expect("nodes not parseable");

    match mode.as_str() {
        "vertex" => {
            union_find(&NodesEdgesMemMapper::new(&name), nodes)
        },
        "hilbert" => {
            union_find(&UpperLowerMemMapper::new(&name), nodes)
        },
        "compressed" => {
            union_find(&DeltaCompressedReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())), nodes)
        },
        _ => { println!("unrecognized mode: {:?}", mode); },
    }
}

fn union_find<G: EdgeMapper>(graph: &G, nodes: u32) {

    let mut roots: Vec<u32> = (0..nodes).collect();      // u32 works, and is smaller than uint/u64
    let mut ranks: Vec<u8> = vec![0u8; nodes as usize];  // u8 should be large enough (n < 2^256)

    graph.map_edges(|mut x, mut y| {

        // x = roots[x as usize];
        // y = roots[y as usize];
        x = unsafe { *roots.get_unchecked(x as usize) };
        y = unsafe { *roots.get_unchecked(y as usize) };

        // while x != roots[x as usize] { x = roots[x as usize]; }
        // while y != roots[y as usize] { y = roots[y as usize]; }
        unsafe { while x != *roots.get_unchecked(x as usize) { x = *roots.get_unchecked(x as usize); } }
        unsafe { while y != *roots.get_unchecked(y as usize) { y = *roots.get_unchecked(y as usize); } }

        if x != y {
            unsafe {
                match ranks[x as usize].cmp(&ranks[y as usize]) {
                    std::cmp::Ordering::Less    => *roots.get_unchecked_mut(x as usize) = y as u32,
                    std::cmp::Ordering::Greater => *roots.get_unchecked_mut(y as usize) = x as u32,
                    std::cmp::Ordering::Equal   => { *roots.get_unchecked_mut(y as usize) = x as u32;
                                                     *ranks.get_unchecked_mut(x as usize) += 1 },
                }
            }
        }

        // works for Hilbert curve order
        // roots[x as usize] = min(x, y);
        // roots[y as usize] = min(x, y);
    });

    let mut non_roots = 0u32;
    for i in 0..roots.len() { if i as u32 != roots[i] { non_roots += 1; }}
    println!("{} non-roots found", non_roots);
}