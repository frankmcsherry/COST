// use std::io::{File, Open, Read, BufferedReader};
// use std::io::fs::stat;
// use std::cmp::min;

use typedrw::TypedMemoryMap;
// use typedrw::TypedReader;

pub trait EdgeMapper {
    fn map_edges<F: FnMut(u32, u32) -> ()>(&self, action: F) -> ();
}

pub trait NodeMapper {
    fn map_nodes<F: FnMut(u32, &[u32]) -> ()>(&self, action: F) -> ();
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
        let mut offset = 0us;
        for &((u16_x, u16_y), count) in self.upper[].iter() {
            let u16_x = (u16_x as u32) << 16;
            let u16_y = (u16_y as u32) << 16;
            for &(l16_x, l16_y) in self.lower[offset .. offset + count as usize].iter() {
                action(u16_x | l16_x as u32, u16_y | l16_y as u32);
            }

            offset += count as usize;
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
        for &(node, count) in self.nodes[].iter() {
            let limit = offset + count as usize;
            for &edge in self.edges[offset..limit].iter() {
                action(node, edge);
            }

            offset = limit;
        }
    }
}

impl NodeMapper for NodesEdgesMemMapper {
    fn map_nodes<F: FnMut(u32, &[u32]) -> ()>(&self, mut action: F) -> () {
        let mut offset = 0us;
        for &(node, count) in self.nodes[].iter() {
            let limit = offset + count as usize;
            action(node, &self.edges[offset..limit]);
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
