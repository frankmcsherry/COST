use hilbert_curve::BytewiseCached;
use typedrw::TypedMemoryMap;

pub trait EdgeMapper {
    fn map_edges<F: FnMut(u32, u32) -> ()>(&self, action: F) -> ();
}

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
    fn map_edges<F: FnMut(u32, u32) -> ()>(&self, mut action: F) -> () {
        let mut offset = 0;
        for &((u16_x, u16_y), count) in self.upper[..].iter() {
            let u16_x = (u16_x as u32) << 16;
            let u16_y = (u16_y as u32) << 16;
            for &(l16_x, l16_y) in self.lower[offset .. offset + count as usize].iter() {
                action(u16_x | l16_x as u32, u16_y | l16_y as u32);
            }

            offset += count as usize;
        }
    }
}

pub struct DeltaCompressedReaderMapper<R: Reader, F: Fn()->R> {
    reader:      F,
}

impl<R: Reader, F: Fn()->R> DeltaCompressedReaderMapper<R, F> {
    pub fn new(reader: F) -> DeltaCompressedReaderMapper<R, F> {
        DeltaCompressedReaderMapper {
            reader: reader,
        }
    }
}

impl<R: Reader, F: Fn()->R> EdgeMapper for DeltaCompressedReaderMapper<R, F> {
    fn map_edges<A: FnMut(u32, u32) -> ()>(&self, mut action: A) -> () {

        let mut hilbert = BytewiseCached::new();
        let mut current = 0u64;
        let mut reader = (self.reader)();

        let mut delta = 0u64;    // for accumulating a delta
        let mut depth = 0u8;     // for counting number of zeros

        let mut buffer = Vec::with_capacity(1 << 16);
        while let Ok(_) = reader.push(1 << 16, &mut buffer) {
            for byte in buffer.drain() {
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
    fn map_edges<F: FnMut(u32, u32) -> ()>(&self, mut action: F) -> () {
        let mut offset = 0;
        for &(node, count) in self.nodes[..].iter() {
            let limit = offset + count as usize;
            for &edge in self.edges[offset..limit].iter() {
                action(node, edge);
            }

            offset = limit;
        }
    }
}

// pub struct UpperLowerMapper {
//     upper_path: Path,
//     lower_path: Path,
// }
//
// impl UpperLowerMapper {
//     pub fn new(graph_name: &str) -> UpperLowerMapper {
//         UpperLowerMapper {
//             upper_path: Path::new(graph_name.to_string() + ".upper"),
//             lower_path: Path::new(graph_name.to_string() + ".lower"),
//         }
//     }
// }
//
// impl EdgeMapper for UpperLowerMapper {
//     fn map_edges<F: FnMut(u32, u32) -> ()>(&self, mut action: F) -> () {
//         let upper_size = stat(&self.upper_path).ok().expect("").size as usize;
//         let upper_file = File::open_mode(&self.upper_path, Open, Read).ok().expect("");
//         let mut upper_reader = BufferedReader::new(upper_file);
//
//         let lower_size = stat(&self.lower_path).ok().expect("").size as usize;
//         let lower_file = File::open_mode(&self.lower_path, Open, Read).ok().expect("");
//         let mut lower_reader = BufferedReader::new(lower_file);
//
//         let mut upper: Vec<_> = (0 .. upper_size / 8).map(|x| ((0u16, 0u16), 0u32)).collect();
//         let upper_read = upper_reader.read_typed_vec(&mut upper).ok().expect("");
//
//         let mut lower: Vec<_> = (0 .. 1 << 20).map(|_| (0u16, 0u16)).collect();
//
//         for &((u16_x, u16_y), mut count) in upper.iter() {
//             let u16_x = (u16_x as u32) << 16;
//             let u16_y = (u16_y as u32) << 16;
//
//             while count > 0 {
//                 let size = min(lower.len(), count as usize);
//                 let lower_read = lower_reader.read_typed_slice(&mut lower[0 .. size]).ok().expect("");
//                 for &(l16_x, l16_y) in lower[0..lower_read].iter() {
//                     action(u16_x + l16_x as u32, u16_y + l16_y as u32);
//                 }
//
//                 count -= lower_read as u32;
//             }
//         }
//     }
// }
