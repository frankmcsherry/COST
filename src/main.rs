#![allow(unstable)]

extern crate alloc;
extern crate core;
extern crate test;

extern crate docopt;
use docopt::Docopt;

use std::cmp::Ordering;
use std::iter::AdditiveIterator;

// use graph_iterator::UpperLowerMapper;
use graph_iterator::{EdgeMapper, UpperLowerMemMapper, NodesEdgesMemMapper};
use hilbert_curve::convert_to_hilbert;

mod typedrw;
mod hilbert_curve;
mod graph_iterator;
mod twitter_parser;

static USAGE: &'static str = "
Usage: COST pagerank  (vertex | hilbert) <path_prefix>
       COST label_prop (vertex | hilbert) <path_prefix>
       COST union_find (vertex | hilbert) <path_prefix>
       COST to_hilbert [--dense] <path_prefix>
";


fn main()
{
    let args = Docopt::new(USAGE).and_then(|dopt| dopt.parse()).unwrap_or_else(|e| e.exit());

    let nodes = 65000000;

    if args.get_bool("vertex") {
        let graph = NodesEdgesMemMapper::new(args.get_str("<path_prefix>"));
        if args.get_bool("pagerank") { pagerank(&graph, nodes, 0.85f32); }
        if args.get_bool("label_prop") { label_propagation(&graph, nodes); }
        if args.get_bool("union_find") { union_find(&graph, nodes); }
    }

    if args.get_bool("hilbert") {
        let graph = UpperLowerMemMapper::new(args.get_str("<path_prefix>"));
        if args.get_bool("pagerank") { pagerank(&graph, nodes, 0.85f32); }
        if args.get_bool("label_prop") { label_propagation(&graph, nodes); }
        if args.get_bool("union_find") { union_find(&graph, nodes); }
    }

    // if args.get_bool("secret") {
    //     let graph = UpperLowerMapper::new(args.get_str("<path_prefix>"));
    //     if args.get_bool("pagerank") { pagerank(&graph, nodes, 0.85f32); }
    //     if args.get_bool("labelprop") { label_propagation(&graph, nodes); }
    //     if args.get_bool("unionfind") { union_find(&graph, nodes); }
    // }

    if args.get_bool("to_hilbert") {
        let graph = NodesEdgesMemMapper::new(args.get_str("<path_prefix>"));
        convert_to_hilbert(&graph, args.get_bool("--dense"), |ux, uy, c, ls| {
            println!("uppers: ({}, {}); count: {}", ux, uy, c);
            for &(lx, ly) in ls.iter(){ println!("\t({}, {})", lx, ly); }
        });
    }
}

fn pagerank<G: EdgeMapper>(graph: &G, nodes: u32, alpha: f32)
{
    let mut src: Vec<f32> = (0..nodes).map(|_| 0f32).collect();
    let mut dst: Vec<f32> = (0..nodes).map(|_| 0f32).collect();
    let mut deg: Vec<f32> = (0..nodes).map(|_| 0f32).collect();

    graph.map_edges(|x, _| { deg[x as usize] += 1f32 });

    for _iteration in (0 .. 10) {
        for node in (0 .. nodes) {
            src[node as usize] = alpha * dst[node as usize] / deg[node as usize];
            dst[node as usize] = 1f32 - alpha;
        }

        graph.map_edges(|x, y| { dst[y as usize] += src[x as usize]; });

        // UNSAFE: graph.map_edges(|x, y| { unsafe { *dst.as_mut_slice().get_unchecked_mut(y) += *src.as_mut_slice().get_unchecked_mut(x); }});
    }
}

fn union_find<G: EdgeMapper>(graph: &G, nodes: u32)
{
    let mut roots: Vec<u32> = (0..nodes).collect();   // u32 works, and is smaller than uint/u64
    let mut ranks: Vec<u8> = (0..nodes).map(|_| 0u8).collect();          // u8 should be large enough (n < 2^256)

    graph.map_edges(|mut x, mut y| {

        x = roots[x as usize];
        y = roots[y as usize];

        // x = unsafe { *roots.as_mut_slice().get_unchecked_mut(x as usize) };
        // y = unsafe { *roots.as_mut_slice().get_unchecked_mut(y as usize) };

        while x != roots[x as usize] { x = roots[x as usize]; }
        while y != roots[y as usize] { y = roots[y as usize]; }

        // unsafe { while x != *roots.as_mut_slice().get_unchecked_mut(x as usize) { x = *roots.as_mut_slice().get_unchecked_mut(x as usize); } }
        // unsafe { while y != *roots.as_mut_slice().get_unchecked_mut(y as usize) { y = *roots.as_mut_slice().get_unchecked_mut(y as usize); } }

        if x != y {
            match ranks[x as usize].cmp(&ranks[y as usize]) {
                Ordering::Less    => roots[x as usize] = y as u32,
                Ordering::Greater => roots[y as usize] = x as u32,
                Ordering::Equal   => { roots[y as usize] = x as u32; ranks[x as usize] += 1 },
            }
        }

        // works for Hilbert curve order
        // roots[x as usize] = min(x, y);
        // roots[y as usize] = min(x, y);
    });

    let mut non_roots = 0u32;
    for i in (0 .. roots.len()) { if i as u32 != roots[i] { non_roots += 1; }}
    println!("{} non-roots found", non_roots);
}

fn label_propagation<G: EdgeMapper>(graph: &G, nodes: u32)
{
    let mut label: Vec<u32> = (0..nodes).collect();

    let mut old_sum = label.iter().map(|x| *x as u64).sum() + 1;
    let mut new_sum = label.iter().map(|x| *x as u64).sum();

    while new_sum < old_sum {
        graph.map_edges(|src, dst| {
            match label[src as usize].cmp(&label[dst as usize]) {
                Ordering::Less    => label[dst as usize] = label[src as usize],
                Ordering::Greater => label[src as usize] = label[dst as usize],
                Ordering::Equal   => { },
            }
        });

        old_sum = new_sum;
        new_sum = label.iter().map(|x| *x as u64).sum();
        println!("iteration");
    }

    let mut non_roots = 0u32;
    for i in (0 .. label.len()) { if i as u32 != label[i] { non_roots += 1; }}
    println!("{} non-roots found", non_roots);
}
