#![allow(non_snake_case)]

extern crate lz4;
extern crate byteorder;

extern crate docopt;
use docopt::Docopt;

extern crate COST;

use std::io::Write;
use std::fs::File;

use COST::hilbert_curve::{encode, Decoder, to_hilbert, merge};
use COST::graph_iterator::ReaderMapper;
use std::io::{BufReader, BufWriter, stdin, stdout};
use byteorder::{WriteBytesExt, LittleEndian};

static USAGE: &'static str = "
Usage: compressed parse_to_hilbert
       compressed merge <source>...
       compressed scan
";

fn main() {
    let args = Docopt::new(USAGE).and_then(|dopt| dopt.parse()).unwrap_or_else(|e| e.exit());

    if args.get_bool("parse_to_hilbert") {
        let reader_mapper = ReaderMapper { reader: || BufReader::new(stdin())};
        let mut writer = BufWriter::new(stdout());

        let mut prev = 0u64;
        to_hilbert(&reader_mapper, |next| {
            assert!(prev < next);
            COST::hilbert_curve::encode(&mut writer, next - prev);
            prev = next;
        });
    }

    if args.get_bool("merge") {
        let mut writer = BufWriter::new(stdout());
        let mut vector = Vec::new();
        for &source in args.get_vec("<source>").iter() {
            vector.push(Decoder::new(lz4::Decoder::new(BufReader::new(File::open(source).unwrap())).unwrap()));
        }

        let mut prev = 0u64;
        merge(vector, |next| {
            assert!(prev <= next);
            if prev < next {
                encode(&mut writer, next - prev);
                prev = next;
            }
        });
    }

    if args.get_bool("scan") {

        let mut bytes = 0u64;
        let mut writer = BufWriter::new(stdout());
        let mut offsets = BufWriter::new(File::create("offsets").unwrap());
        let mut buffer = Vec::new();

        let mut prev_edge = 0u64;
        let mut prev_node = 0u64;

        for next in Decoder::new(BufReader::new(stdin())) {

            let node = next >> 32;
            let edge = next % (1 << 32);

            while prev_node < node {
                offsets.write_u64::<LittleEndian>(bytes).unwrap();
                prev_node += 1;
                prev_edge = 0u64;
            }

            let mut diff = edge - prev_edge;

            while diff > 127 {
                buffer.push(((diff & 127) as u8) + 128u8);
                diff = diff >> 7;
                bytes += 1;
            }
            buffer.push(diff as u8);
            bytes += 1;

            if buffer.len() > (1 << 20) {
                writer.write_all(&buffer[..]).unwrap();
                buffer.clear();
            }

            prev_edge = edge;
        }

        writer.write_all(&buffer[..]).unwrap();
        buffer.clear();
    }
}
