use std::io::{BufRead, BufReader, BufWriter};
use std::fs::File;
use graph_iterator::EdgeMapper;
use byteorder::{WriteBytesExt, LittleEndian};

// source should be something like "path/to/twitter_rv.net"
// target should be so that you don't mind having "target.nodes" and "target.edges" clobbered.
pub fn _parse_to_vertex(source: &str, target: &str) {
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

pub struct ReaderMapper<B: BufRead, F: Fn() -> B> {
    pub reader: F,
}

impl<R:BufRead, RF: Fn() -> R> EdgeMapper for ReaderMapper<R, RF> {
    fn map_edges<F: FnMut(u32, u32) -> ()>(&self, mut action: F) -> () {
        let reader = (self.reader)();
        for readline in reader.lines() {
            let line = readline.ok().expect("read error");
            let elts: Vec<&str> = line[..].split_whitespace().collect();
            let src: u32 = elts[0].parse().ok().expect("malformed src");
            let dst: u32 = elts[1].parse().ok().expect("malformed dst");
            action(src, dst);
        }
    }
}
