use std::io::{File, Open, Read, BufferedReader, Write, BufferedWriter, IoResult};

// source should be something like "path/to/twitter_rv.net"
// target should be so that you don't mind having "target.nodes" and "target.edges" clobbered.
fn _parse_twitter(source: &str, target: &str) -> IoResult<()> {
    let p = Path::new(source);
    let pn = Path::new(format!("{}.nodes", target));
    let pe = Path::new(format!("{}.edges", target));

    let mut reader = BufferedReader::new(try!(File::open_mode(&p, Open, Read)));
    let mut edge_writer = BufferedWriter::new(try!(File::open_mode(&pe, Open, Write)));
    let mut node_writer = BufferedWriter::new(try!(File::open_mode(&pn, Open, Write)));

    let line = try!(reader.read_line());
    let elts: Vec<&str> = line.as_slice().words().collect();

    let mut src: u32 = elts[0].parse().expect("malformed src");
    let mut dst: u32 = elts[1].parse().expect("malformed dst");

    try!(edge_writer.write_le_u32(dst));
    let mut ctr: u32 = 1;

    for readline in reader.lines() {
        let line = try!(readline);

        let elts: Vec<&str> = line.as_slice().words().collect();

        let read_src: u32 = elts[0].parse().expect("malformed src");
        let read_dst: u32 = elts[1].parse().expect("malformed dst");

        if read_src != src {
            try!(node_writer.write_le_u32(src));
            try!(node_writer.write_le_u32(ctr));
            ctr = 0;
        }

        try!(edge_writer.write_le_u32(dst));
        ctr = ctr + 1;

        src = read_src;
        dst = read_dst;
    }

    try!(node_writer.write_le_u32(src));
    try!(node_writer.write_le_u32(ctr));

    return Ok(());
}
