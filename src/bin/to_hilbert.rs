extern crate byteorder;
extern crate COST;

use std::fs::File;
use std::io::BufWriter;
use byteorder::{WriteBytesExt, LittleEndian};
use COST::graph_iterator::NodesEdgesMemMapper;

fn main() {

    if std::env::args().len() != 2 && std::env::args().len() != 3 {
        println!("Usage: to_hilbert <prefix> [--dense]");
        println!("NOTE: <prefix>.upper and <prefix>.lower will be overwritten.");
        return;
    }

    let prefix = std::env::args().nth(1).expect("name unavailable");
    let dense = std::env::args().nth(2) == Some("--dense".to_string());

    let graph = NodesEdgesMemMapper::new(&prefix);
    let mut u_writer = BufWriter::new(File::create(format!("{}.upper", prefix)).unwrap());
    let mut l_writer = BufWriter::new(File::create(format!("{}.lower", prefix)).unwrap());

    COST::hilbert_curve::convert_to_hilbert(&graph, dense, |ux, uy, c, ls| {
        u_writer.write_u16::<LittleEndian>(ux).unwrap();
        u_writer.write_u16::<LittleEndian>(uy).unwrap();
        u_writer.write_u32::<LittleEndian>(c).unwrap();
        for &(lx, ly) in ls.iter(){
            l_writer.write_u16::<LittleEndian>(lx).unwrap();
            l_writer.write_u16::<LittleEndian>(ly).unwrap();
        }
    });
}