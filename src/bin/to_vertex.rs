extern crate byteorder;
extern crate COST;

use std::io::{BufReader, BufWriter};
use std::fs::File;
use COST::graph_iterator::{EdgeMapper, ReaderMapper};
use byteorder::{WriteBytesExt, LittleEndian};

fn main() {

    if std::env::args().len() != 3 {
        println!("Usage: to_vertex <source> <prefix>");
        println!("NOTE: <prefix>.nodes and <prefix>.edges will be overwritten.");
        return;
    }

    let source = std::env::args().nth(1).expect("source unavailable"); let source = &source;
    let target = std::env::args().nth(2).expect("prefix unavailable"); let target = &target;

    let reader_mapper = ReaderMapper { reader: || BufReader::new(File::open(source).unwrap()) };

    let mut edge_writer = BufWriter::new(File::create(format!("{}.edges", target)).unwrap());
    let mut node_writer = BufWriter::new(File::create(format!("{}.nodes", target)).unwrap());

    let mut cnt = 0;
    let mut src = 0;

    reader_mapper.map_edges(|x, y| {
        if x != src {
            if cnt > 0 {
                node_writer.write_u32::<LittleEndian>(src).ok().expect("write error");
                node_writer.write_u32::<LittleEndian>(cnt).ok().expect("write error");
                cnt = 0;
            }
            src = x;
        }

        edge_writer.write_u32::<LittleEndian>(y).ok().expect("write error");
        cnt += 1;
    });

    if cnt > 0 {
        node_writer.write_u32::<LittleEndian>(src).ok().expect("write error");
        node_writer.write_u32::<LittleEndian>(cnt).ok().expect("write error");
    }
}