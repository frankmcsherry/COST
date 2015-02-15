use std::old_io::{File, Open, Read, BufferedReader, Write, BufferedWriter};
use graph_iterator::EdgeMapper;

// source should be something like "path/to/twitter_rv.net"
// target should be so that you don't mind having "target.nodes" and "target.edges" clobbered.
pub fn _parse_to_vertex(source: &str, target: &str) {
    let reader_mapper = ReaderMapper {
        reader: || BufferedReader::new(File::open_mode(&Path::new(source), Open, Read).unwrap())
    };

    let mut edge_writer = BufferedWriter::new(File::open_mode(&Path::new(format!("{}.edges", target)), Open, Write).ok().expect("err"));
    let mut node_writer = BufferedWriter::new(File::open_mode(&Path::new(format!("{}.nodes", target)), Open, Write).ok().expect("err"));

    let mut cnt = 0;
    let mut src = 0;

    reader_mapper.map_edges(|x, y| {
        if x != src {
            if cnt > 0 {
                node_writer.write_le_u32(src).ok().expect("write error");
                node_writer.write_le_u32(cnt).ok().expect("write error");
                cnt = 0;
            }
            src = x;
        }

        edge_writer.write_le_u32(y).ok().expect("write error");
        cnt += 1;
    });

    if cnt > 0 {
        node_writer.write_le_u32(src).ok().expect("write error");
        node_writer.write_le_u32(cnt).ok().expect("write error");
    }
}

pub struct ReaderMapper<B: Buffer, F: Fn() -> B> {
    pub reader: F,
}

impl<R:Buffer, RF: Fn() -> R> EdgeMapper for ReaderMapper<R, RF> {
    fn map_edges<F: FnMut(u32, u32) -> ()>(&self, mut action: F) -> () {
        let mut reader = (self.reader)();
        for readline in reader.lines() {
            let line = readline.ok().expect("read error");
            let elts: Vec<&str> = line.as_slice().words().collect();
            let src: u32 = elts[0].parse().ok().expect("malformed src");
            let dst: u32 = elts[1].parse().ok().expect("malformed dst");
            action(src, dst);
        }
    }
}
