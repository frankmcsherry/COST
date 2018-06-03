extern crate COST;

use std::fs::File;

use COST::graph_iterator::{EdgeMapper, DeltaCompressedReaderMapper, NodesEdgesMemMapper, UpperLowerMemMapper };
use std::io::BufReader;

fn main() {

    if std::env::args().len() != 4 {
        println!("Usage: bfs  (vertex | hilbert | compressed) <prefix> nodes");
        return;
    }

    let mode = std::env::args().nth(1).expect("mode unavailable");
    let name = std::env::args().nth(2).expect("name unavailable");
    let nodes: u32 = std::env::args().nth(3).expect("nodes unavailable").parse().expect("nodes not parseable");

    match mode.as_str() {
        "vertex" => {
            bfs(&NodesEdgesMemMapper::new(&name), nodes)
        },
        "hilbert" => {
            bfs(&UpperLowerMemMapper::new(&name), nodes)
        },
        "compressed" => {
            bfs(&DeltaCompressedReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())), nodes)
        },
        _ => { println!("unrecognized mode: {:?}", mode); },
    }
}

// NOTE : The following code is specific to the common crawl 2012 dataset.
// NOTE : It may behave very badly indeed with other datasets.

fn bfs<G: EdgeMapper>(graph: &G, nodes: u32) {

    let timer = std::time::Instant::now();

    // let nodes = 3_563_602_788 + 1;

    let mut roots: Vec<u32> = (0..nodes).collect();

    let mut label = vec![65535u16; nodes as usize];
    label[0] = 0;

    graph.map_edges(|mut x, mut y| {

        if x == 0 { label[y as usize] = 1; }
        if y == 0 { label[x as usize] = 1; }

        x = unsafe { *roots.get_unchecked(x as usize) };
        y = unsafe { *roots.get_unchecked(y as usize) };

        unsafe { while x != *roots.get_unchecked(x as usize) { x = *roots.get_unchecked(x as usize); } }
        unsafe { while y != *roots.get_unchecked(y as usize) { y = *roots.get_unchecked(y as usize); } }

        // works for Hilbert curve order
        roots[x as usize] = ::std::cmp::min(x, y);
        roots[y as usize] = ::std::cmp::min(x, y);
    });

    for i in 1..nodes {
        let mut node = i;
        while node != roots[node as usize] { node = roots[node as usize]; }
        if node != 0 { label[i as usize] = 0; }
    }

    let mut roots = Vec::with_capacity(nodes as usize);

    for i in 1..nodes {
        if label[i as usize] == 1 { roots.push((i,0)); }
        // else                      { roots[i as usize] = i; }
    }

    println!("{:?}\titeration: {}", timer.elapsed(), 0);

    // WTF is this? What are YOU PLANNNING?!??!
    let mut edges = Vec::new();
    let mut iteration = 1;

    // iterate as long as there are changes
    while edges.len() == edges.capacity() {

        // allocate if the first iteration, clear otherwise
        if edges.capacity() == 0 { edges = Vec::with_capacity(1 << 30); }
        else                     { edges.clear(); }

        graph.map_edges(|src, dst| {

            let label_src = unsafe { *label.get_unchecked(src as usize) };
            let label_dst = unsafe { *label.get_unchecked(dst as usize) };

            if edges.len() < edges.capacity() {

                if (label_src > iteration && label_dst > iteration + 1) ||
                   (label_dst > iteration && label_src > iteration + 1) {
                    edges.push((src, dst));
                }
            }

            if label_src == iteration && label_dst > iteration + 1 {
                unsafe { *label.get_unchecked_mut(dst as usize) = iteration + 1; }
                roots.push((dst, src));
            }

            if label_dst == iteration && label_src > iteration + 1 {
                unsafe { *label.get_unchecked_mut(src as usize) = iteration + 1; }
                roots.push((src, dst));
            }
        });

        iteration += 1;
        println!("{:?}\titeration: {}", timer.elapsed(), iteration);
    }

    let mut done = false;
    while !done {
        done = true;
        edges.retain(|&(src,dst)| {

            if label[src as usize] == iteration && label[dst as usize] > iteration + 1 {
                label[dst as usize] = iteration + 1;
                roots.push((dst, src));
                done = false;
            }
            else
            if label[dst as usize] == iteration && label[src as usize] > iteration + 1 {
                label[src as usize] = iteration + 1;
                roots.push((src, dst));
                done = false;
            }

            (label[src as usize] > iteration && label[dst as usize] > iteration + 1) ||
            (label[dst as usize] > iteration && label[src as usize] > iteration + 1)
        });

        iteration += 1;
        println!("{:?}\titeration: {}", timer.elapsed(), iteration);
    }

    let mut counts = vec![0u64; 1 << 16];
    for &x in &label { counts[x as usize] += 1; }
    for (dist, count) in counts.iter().enumerate() {
        if *count > 0 {
            println!("counts[{}]: {}", dist, count);
        }
    }
}