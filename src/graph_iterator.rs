use std::io::Read;
use hilbert_curve::BytewiseCached;
use typedrw::TypedMemoryMap;

pub trait EdgeMapper {
    fn map_edges(&self, action: impl FnMut(u32, u32));
}

pub struct DeltaCompressedReaderMapper<R: Read, F: Fn()->R> {
    reader: F,
}

impl<R: Read, F: Fn()->R> DeltaCompressedReaderMapper<R, F> {
    pub fn new(reader: F) -> DeltaCompressedReaderMapper<R, F> {
        DeltaCompressedReaderMapper {
            reader: reader,
        }
    }
}

impl<R: Read, F: Fn()->R> EdgeMapper for DeltaCompressedReaderMapper<R, F> {
    fn map_edges(&self, mut action: impl FnMut(u32, u32)) {

        let mut hilbert = BytewiseCached::new();
        let mut current = 0u64;
        let mut reader = (self.reader)();

        let mut delta = 0u64;    // for accumulating a delta
        let mut depth = 0u8;     // for counting number of zeros

        let mut buffer = vec![0u8; 1 << 16];
        while let Ok(read) = reader.read(&mut buffer[..]) {
            if read == 0 {
                // Reached EOF.
                break;
            }
            for &byte in &buffer[..read] {
                if byte == 0 && delta == 0 {
                    depth += 1;
                }
                else {
                    delta = (delta << 8) + (byte as u64);
                    if depth == 0 {
                        current += delta;
                        delta = 0;
                        let (x,y) = hilbert.detangle(current);
                        action(x,y);
                    }
                    else {
                        depth -= 1;
                    }
                }
            }
        }
    }
}

pub struct DeltaCompressedSliceMapper<'a> {
    slice: &'a [u8],
}

impl<'a> DeltaCompressedSliceMapper<'a> {
    pub fn new(slice: &'a [u8]) -> DeltaCompressedSliceMapper<'a> {
        DeltaCompressedSliceMapper {
            slice: slice,
        }
    }
}

impl<'a> EdgeMapper for DeltaCompressedSliceMapper<'a> {
    fn map_edges(&self, mut action: impl FnMut(u32, u32)) {

        let mut hilbert = BytewiseCached::new();
        let mut current = 0u64;

        let mut cursor = 0;
        while cursor < self.slice.len() {
            let byte = unsafe { *self.slice.get_unchecked(cursor) };
            cursor += 1;

            if byte > 0 {
                current += byte as u64;
                let (x,y) = hilbert.detangle(current);
                action(x,y);
            }
            else {
                let mut depth = 2;
                while unsafe { *self.slice.get_unchecked(cursor) } == 0 {
                    cursor += 1;
                    depth += 1;
                }
                let mut delta = 0u64;
                while depth > 0 {
                    delta = (delta << 8) + (unsafe { *self.slice.get_unchecked(cursor) } as u64);
                    cursor += 1;
                    depth -= 1;
                }

                current += delta;
                let (x,y) = hilbert.detangle(current);
                action(x,y);
            }
        }
    }
}

// // naughty method using unsafe transmute to read a filled binary buffer as a typed buffer
// fn read_as_typed<'a, R: Read, T: Copy>(reader: &mut R, buffer: &'a mut[u8]) -> Result<&'a[T]> {
//     if mem::size_of::<T>() * (buffer.len() / mem::size_of::<T>()) < buffer.len() {
//         panic!("buffer size must be a multiple of mem::size_of::<T>() = {:?}", mem::size_of::<T>());
//     }
//
//     let mut read = try!(reader.read(buffer));
//     while mem::size_of::<T>() * (read / mem::size_of::<T>()) < read {
//         read += try!(reader.read(&mut buffer[read..]));
//     }
//
//     Ok(unsafe { mem::transmute(RawSlice {
//         data: buffer.as_mut_ptr() as *const T,
//         len:  read / mem::size_of::<T>(),
//     }) })
// }

// pub struct UpperLowerMapper<R1: Read, R2: Read, F1: Fn()->R1, F2: Fn()->R2> {
//     pub upper: F1,
//     pub lower: F2,
// }
//
// impl<R1: Read, R2: Read, F1: Fn()->R1, F2: Fn()->R2> EdgeMapper for UpperLowerMapper<R1, R2, F1, F2> {
//     fn map_edges<F: FnMut(u32, u32) -> ()>(&self, mut action: F) -> () {
//         let mut upper_reader = (self.upper)();
//         let mut lower_reader = (self.lower)();
//         let mut upper_buffer = vec![0u8; 1 << 20];
//         let mut lower_buffer = vec![0u8; 1 << 20];
//         while let Ok(upper) = read_as_typed::<_,((u16,u16),u32)>(&mut upper_reader, &mut upper_buffer[..]) {
//             for &((ux, uy), mut count) in upper {
//                 let ux = (ux as u32) << 16;
//                 let uy = (uy as u32) << 16;
//                 while count > 0 {
//                     let size = min(lower_buffer.len(), 4 * count as usize);
//                     if let Ok(lower) = read_as_typed::<_,(u16,u16)>(&mut lower_reader, &mut lower_buffer[..size]) {
//                         for &(lx, ly) in lower {
//                             action(ux + lx as u32, uy + ly as u32);
//                         }
//                         count -= lower.len() as u32;
//                     }
//                 }
//             }
//         }
//     }
// }

// pub struct NodesEdgesMapper<R1: Read, R2: Read, F1: Fn()->R1, F2: Fn()->R2> {
//     pub nodes: F1,
//     pub edges: F2,
// }
//
// impl<R1: Read, R2: Read, F1: Fn()->R1, F2: Fn()->R2> EdgeMapper for NodesEdgesMapper<R1, R2, F1, F2> {
//     fn map_edges<F: FnMut(u32, u32) -> ()>(&self, mut action: F) -> () {
//         let mut nodes_reader = (self.nodes)();
//         let mut edges_reader = (self.edges)();
//         let mut nodes_buffer = vec![0u8; 1 << 20];
//         let mut edges_buffer = vec![0u8; 1 << 20];
//         while let Ok(nodes) = read_as_typed::<_,(u32,u32)>(&mut nodes_reader, &mut nodes_buffer[..]) {
//             for &(source, mut count) in nodes {
//                 while count > 0 {
//                     let size = min(edges_buffer.len(), 4 * count as usize);
//                     if let Ok(edges) = read_as_typed::<_,u32>(&mut edges_reader, &mut edges_buffer[..size]) {
//                         for &target in edges {
//                             action(source, target);
//                         }
//                         count -= edges.len() as u32;
//                     }
//                 }
//             }
//         }
//     }
// }

pub struct UpperLowerMemMapper {
    upper:  TypedMemoryMap<((u16,u16), u32)>,
    lower:  TypedMemoryMap<(u16, u16)>,
}

impl UpperLowerMemMapper {
    pub fn new(graph_name: &str) -> UpperLowerMemMapper {
        UpperLowerMemMapper {
            upper: TypedMemoryMap::new(format!("{}.upper", graph_name)),
            lower: TypedMemoryMap::new(format!("{}.lower", graph_name)),
        }
    }
}

impl EdgeMapper for UpperLowerMemMapper {
    fn map_edges(&self, mut action: impl FnMut(u32, u32)) {
        let mut slice = &self.lower[..];
        for &((u16_x, u16_y), count) in &self.upper[..] {
            let u16_x = (u16_x as u32) << 16;
            let u16_y = (u16_y as u32) << 16;
            for &(l16_x, l16_y) in &slice[.. count as usize] {
                action(u16_x | l16_x as u32, u16_y | l16_y as u32);
            }

            slice = &slice[count as usize ..];
        }
    }
}

pub struct NodesEdgesMemMapper {
    nodes:  TypedMemoryMap<(u32, u32)>,
    edges:  TypedMemoryMap<u32>,
}

impl NodesEdgesMemMapper {
    pub fn new(graph_name: &str) -> NodesEdgesMemMapper {
        NodesEdgesMemMapper {
            nodes: TypedMemoryMap::new(format!("{}.nodes", graph_name)),
            edges: TypedMemoryMap::new(format!("{}.edges", graph_name)),
        }
    }
}

impl EdgeMapper for NodesEdgesMemMapper {
    fn map_edges(&self, mut action: impl FnMut(u32, u32)) {
        let mut slice = &self.edges[..];
        for &(node, count) in &self.nodes[..] {
            for &edge in &slice[.. count as usize] {
                action(node, edge);
            }

            slice = &slice[count as usize ..];
        }
    }
}

pub struct ReaderMapper<B: ::std::io::BufRead, F: Fn() -> B> {
    pub reader: F,
}

impl<R: ::std::io::BufRead, RF: Fn() -> R> EdgeMapper for ReaderMapper<R, RF> {
    fn map_edges(&self, mut action: impl FnMut(u32, u32)) {
        let reader = (self.reader)();
        for readline in reader.lines() {
            let line = readline.ok().expect("read error");
            if !line.starts_with('#') {
                let mut elts = line[..].split_whitespace();
                let src: u32 = elts.next().unwrap().parse().ok().expect("malformed src");
                let dst: u32 = elts.next().unwrap().parse().ok().expect("malformed dst");
                action(src, dst);
            }
        }
    }
}