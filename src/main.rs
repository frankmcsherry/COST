#![feature(old_io)]
#![feature(core)]
#![feature(collections)]
#![feature(old_path)]
#![feature(os)]
#![feature(test)]
#![feature(alloc)]
#![feature(std_misc)]
#![feature(str_words)]

extern crate alloc;
extern crate core;
extern crate test;

extern crate docopt;
use docopt::Docopt;

use std::cmp::Ordering;
use std::iter::AdditiveIterator;

use std::old_io::{File, Open, Write, Read, BufferedWriter};
use std::old_io::stdio::{stdin, stdout};
use std::old_io::BufferedReader;

// use graph_iterator::UpperLowerMapper;
use graph_iterator::{EdgeMapper, UpperLowerMemMapper, DeltaCompressedReaderMapper, NodesEdgesMemMapper};
use hilbert_curve::{encode, Decoder, convert_to_hilbert, BytewiseHilbert, to_hilbert, merge};
use twitter_parser::{ ReaderMapper };

mod typedrw;
mod hilbert_curve;
mod graph_iterator;
mod twitter_parser;

static USAGE: &'static str = "
Usage: COST stats  (vertex | hilbert | compressed) <prefix>
       COST print  (vertex | hilbert | compressed) <prefix>
       COST pagerank  (vertex | hilbert | compressed) <prefix>
       COST label_prop (vertex | hilbert | compressed) <prefix>
       COST union_find (vertex | hilbert | compressed) <prefix>
       COST to_hilbert [--dense] <prefix>
       COST parse_to_hilbert
       COST merge <source>...
";


fn main()
{
    let args = Docopt::new(USAGE).and_then(|dopt| dopt.parse()).unwrap_or_else(|e| e.exit());

    let nodes = 3563602788 + 1; //65000000;

    if args.get_bool("vertex") {
        let graph = NodesEdgesMemMapper::new(args.get_str("<prefix>"));
        if args.get_bool("stats") { stats(&graph); }
        if args.get_bool("print") { print(&graph); }
        if args.get_bool("pagerank") { pagerank(&graph, nodes, 0.85f32); }
        if args.get_bool("label_prop") { label_propagation(&graph, nodes); }
        if args.get_bool("union_find") { union_find(&graph, nodes); }
    }

    if args.get_bool("hilbert") {
        let graph = UpperLowerMemMapper::new(args.get_str("<prefix>"));
        if args.get_bool("stats") { stats(&graph); }
        if args.get_bool("print") { print(&graph); }
        if args.get_bool("pagerank") { pagerank(&graph, nodes, 0.85f32); }
        if args.get_bool("label_prop") { label_propagation(&graph, nodes); }
        if args.get_bool("union_find") { union_find(&graph, nodes); }
    }

    if args.get_bool("compressed") {
        let graph = DeltaCompressedReaderMapper::new(|| BufferedReader::new(File::open_mode(&Path::new(args.get_str("<prefix>")), Open, Read)));
        if args.get_bool("stats") { stats(&graph); }
        if args.get_bool("print") { print(&graph); }
        if args.get_bool("pagerank") { pagerank(&graph, nodes, 0.85f32); }
        if args.get_bool("label_prop") { label_propagation(&graph, nodes); }
        if args.get_bool("union_find") { union_find(&graph, nodes); }
    }
    // if args.get_bool("secret") {
    //     let graph = UpperLowerMapper::new(args.get_str("<prefix>"));
    //     if args.get_bool("pagerank") { pagerank(&graph, nodes, 0.85f32); }
    //     if args.get_bool("labelprop") { label_propagation(&graph, nodes); }
    //     if args.get_bool("unionfind") { union_find(&graph, nodes); }
    // }

    if args.get_bool("to_hilbert") {
        let prefix = args.get_str("<prefix>");
        let graph = NodesEdgesMemMapper::new(prefix);

        let mut u_writer = BufferedWriter::new(File::open_mode(&Path::new(format!("{}.upper", prefix)), Open, Write).ok().expect("err"));
        let mut l_writer = BufferedWriter::new(File::open_mode(&Path::new(format!("{}.lower", prefix)), Open, Write).ok().expect("err"));

        convert_to_hilbert(&graph, args.get_bool("--dense"), |ux, uy, c, ls| {
            u_writer.write_le_u16(ux).ok().expect("err");
            u_writer.write_le_u16(uy).ok().expect("err");
            u_writer.write_le_u32(c).ok().expect("err");
            for &(lx, ly) in ls.iter(){
                l_writer.write_le_u16(lx).ok().expect("err");
                l_writer.write_le_u16(ly).ok().expect("err");
            }
        });
    }

    if args.get_bool("parse_to_hilbert") {
        let reader_mapper = ReaderMapper { reader: || BufferedReader::new(stdin())};
        let mut writer = BufferedWriter::new(stdout());

        let mut prev = 0u64;
        to_hilbert(&reader_mapper, |next| {
            assert!(prev < next);
            encode(&mut writer, next - prev);
            prev = next;
        });
    }

    if args.get_bool("merge") {
        let mut writer = BufferedWriter::new(stdout());
        let mut vector = Vec::new();
        for &source in args.get_vec("<source>").iter() {
            vector.push(Decoder::new(BufferedReader::new(File::open_mode(&Path::new(source), Open, Read))));
        }

        let mut prev = 0u64;
        merge(vector, |next| {
            assert!(prev < next);
            encode(&mut writer, next - prev);
            prev = next;
        });
    }
}

fn stats<G: EdgeMapper>(graph: &G) {
    let mut max_x = 0;
    let mut max_y = 0;
    let mut edges = 0;
    graph.map_edges(|x, y| {
        if max_x < x { max_x = x; }
        if max_y < y { max_y = y; }
        edges += 1;
    });

    println!("max x: {}", max_x);
    println!("max y: {}", max_y);
    println!("edges: {}", edges);
}

fn print<G: EdgeMapper>(graph: &G) {
    let hilbert = BytewiseHilbert::new();
    graph.map_edges(|x, y| { println!("{}\t{} -> {}", x, y, hilbert.entangle((x,y))) });
}

fn pagerank<G: EdgeMapper>(graph: &G, nodes: u32, alpha: f32)
{
    let mut src: Vec<f32> = (0..nodes).map(|_| 0f32).collect();
    let mut dst: Vec<f32> = (0..nodes).map(|_| 0f32).collect();
    let mut deg: Vec<f32> = (0..nodes).map(|_| 0f32).collect();

    graph.map_edges(|x, _| { deg[x as usize] += 1f32 });

    for _iteration in (0 .. 20) {
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
