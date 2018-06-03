use std::io::{Read, Write};
use std::collections::HashMap;
use graph_iterator::EdgeMapper;
use byteorder::{ReadBytesExt, WriteBytesExt};

#[inline]
pub fn encode<W: Write>(writer: &mut W, diff: u64) {
    assert!(diff > 0);
    for &shift in [56, 48, 40, 32, 24, 16, 8].iter() {
        if (diff >> shift) != 0 {
            writer.write_u8(0u8).ok().expect("write error");
        }
    }
    for &shift in [56, 48, 40, 32, 24, 16, 8].iter() {
        if (diff >> shift) != 0 {
            writer.write_u8((diff >> shift) as u8).ok().expect("write error");
        }
    }
    writer.write_u8(diff as u8).ok().expect("write error");
}

#[inline]
pub fn decode<R: Read>(reader: &mut R) -> Option<u64> {
    if let Ok(mut read) = reader.read_u8() {
        let mut count = 0u64;
        while read == 0 {
            count += 1;
            read = reader.read_u8().unwrap();
        }

        let mut diff = read as u64;
        for _ in 0..count {
            diff = (diff << 8) + (reader.read_u8().unwrap() as u64);
        }

        Some(diff)
    }
    else { None }
}

#[test]
fn test_encode_decode() {
    let test_vec = vec![1, 2, 1 << 20, 1 << 60];
    let mut writer = Vec::new();
    for &elt in test_vec.iter() {
        encode(&mut writer, elt);
    }

    let mut test_out = Vec::new();
    let mut reader = &writer[..];
    while let Some(elt) = decode(&mut reader) {
        test_out.push(elt);
    }

    assert_eq!(test_vec, test_out);
}

pub struct Decoder<R: Read> {
    reader:     R,
    current:    u64,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Decoder<R> {
        Decoder { reader: reader, current: 0 }
    }
}

impl<R: Read> Iterator for Decoder<R> {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        if let Some(diff) = decode(&mut self.reader) {
            assert!(self.current < self.current + diff);
            self.current += diff;
            Some(self.current)
        }
        else { None }
    }
}

pub fn to_hilbert<I, O>(graph: &I, mut output: O) -> ()
where I : EdgeMapper,
      O : FnMut(u64)->(),
{
    let hilbert = BytewiseHilbert::new();
    let mut buffer = Vec::new();
    graph.map_edges(|node, edge| { buffer.push(hilbert.entangle((node, edge))); });
    buffer.sort();
    for &element in buffer.iter() { output(element); }
}

pub fn convert_to_hilbert<I, O>(graph: &I, make_dense: bool, mut output: O) -> ()
where I : EdgeMapper,
      O : FnMut(u16, u16, u32, &Vec<(u16, u16)>) -> (),
{
    let mut uppers: HashMap<u32,Vec<u32>> = HashMap::new();
    let mut names = Vec::new();
    let mut names_count = 0i32;
    let hilbert = BytewiseHilbert::new();

    graph.map_edges(|mut node, mut edge| {
        if make_dense {
            while names.len() as u32 <= node { names.push(-1i32); }
            while names.len() as u32 <= edge { names.push(-1i32); }
            if names[node as usize] == -1i32 { names[node as usize] = names_count; names_count += 1; }
            if names[edge as usize] == -1i32 { names[edge as usize] = names_count; names_count += 1; }

            node = names[node as usize] as u32;
            edge = names[edge as usize] as u32;
        }

        let entangled = hilbert.entangle((node as u32, edge as u32));
        let upper = (entangled >> 32) as u32;
        let lower = entangled as u32;

        uppers.entry(upper).or_insert(Vec::new()).push(lower);
    });

    let mut keys: Vec<u32> = uppers.keys().map(|x|x.clone()).collect();
    keys.sort();

    let mut temp = Vec::new();
    for &upper in keys.iter() {
        let mut lowers = uppers.remove(&upper).unwrap();
        if lowers.len() > 0 {
            let upair = hilbert.detangle((upper as u64) << 32);
            let upperx = (upair.0 >> 16) as u16;
            let uppery = (upair.1 >> 16) as u16;
            let length = lowers.len() as u32;

            lowers.sort();  // TODO : Check Radix sort perf
            temp.clear();

            for &lower in lowers.iter() {
                let lpair = hilbert.detangle(((upper as u64) << 32) + (lower as u64));
                let lowerx = (lpair.0 & 65535u32) as u16;
                let lowery = (lpair.1 & 65535u32) as u16;
                temp.push((lowerx, lowery));
            }

            output(upperx, uppery, length, &temp);
        }
    }
}

pub fn merge<I: Iterator<Item=u64>, O: FnMut(u64)->()>(mut iterators: Vec<I>, mut output: O) {
    let mut values = Vec::new();
    for iterator in iterators.iter_mut() { values.push(iterator.next()); }

    let mut val_old = 0;
    let mut done = false;
    while !done {
        let mut arg_min = iterators.len();
        let mut val_min = 0u64;
        for (index, &value) in values.iter().enumerate() {
            if let Some(val) = value {
                if arg_min > index || val < val_min {
                    arg_min = index;
                    val_min = val;
                    // done = false;
                }
            }
        }

        if arg_min < iterators.len() {
            values[arg_min] = iterators[arg_min].next();
            if let Some(val) = values[arg_min] {
                assert!(val > val_min);
            }
            assert!(val_old <= val_min);
            val_old = val_min;
            output(val_min);
        }
        else {
            done = true;
        }
    }

    // confirm that we haven't left anything behind
    assert!(!values.iter().any(|x|x.is_some()));
}

// algorithm drawn in large part from http://en.wikipedia.org/wiki/Hilbert_curve
// bytewise implementation based on tracking cumulative rotation / mirroring.

pub struct BytewiseCached {
    hilbert:    BytewiseHilbert,
    prev_hi:    u64,
    prev_out:   (u32, u32),
    prev_rot:   (bool, bool),
}

impl BytewiseCached {
    #[inline(always)]
    pub fn detangle(&mut self, tangle: u64) -> (u32, u32) {
        let (mut x_byte, mut y_byte) = unsafe { *self.hilbert.detangle.get_unchecked(tangle as u16 as usize) };

        // validate self.prev_rot, self.prev_out
        if self.prev_hi != (tangle >> 16) {
            self.prev_hi = tangle >> 16;

            // detangle with a bit set to see what happens to it
            let low = 255; //self.hilbert.entangle((0xF, 0)) as u16;
            let (x, y) = self.hilbert.detangle((self.prev_hi << 16) + low as u64);

            let value = (x as u8, y as u8);
            self.prev_rot = match value {
                (0x0F, 0x00) => (false, false), // nothing
                (0x00, 0x0F) => (true, false),  // swapped
                (0xF0, 0xFF) => (false, true),  // flipped
                (0xFF, 0xF0) => (true, true),   // flipped & swapped
                val => panic!(format!("Found : ({:x}, {:x})", val.0, val.1)),
            };
            self.prev_out = (x & 0xFFFFFF00, y & 0xFFFFFF00);
        }


        if self.prev_rot.1 {
            x_byte = 255 - x_byte;
            y_byte = 255 - y_byte;
        }
        if self.prev_rot.0 {
            let temp = x_byte; x_byte = y_byte; y_byte = temp;
        }

        return (self.prev_out.0 + x_byte as u32, self.prev_out.1 + y_byte as u32);
    }
    pub fn new() -> BytewiseCached {
        let mut result = BytewiseCached {
            hilbert: BytewiseHilbert::new(),
            prev_hi: 0xFFFFFFFFFFFFFFFF,
            prev_out: (0,0),
            prev_rot: (false, false),
        };

        result.detangle(0); // ensures that we set the cached stuff correctly
        return result;
    }
}

pub struct BytewiseHilbert {
    entangle: Vec<u16>,         // entangle[x_byte << 16 + y_byte] -> tangle
    detangle: Vec<(u8, u8)>,    // detangle[tangle] -> (x_byte, y_byte)
    rotation: Vec<u8>,          // info on rotation, keyed per self.entangle
}

impl BytewiseHilbert {
    pub fn new() -> BytewiseHilbert {
        let mut entangle = Vec::new();
        let mut detangle: Vec<_> = (0..65536).map(|_| (0u8, 0u8)).collect();
        let mut rotation = Vec::new();
        for x in 0u32..256 {
            for y in 0u32..256 {
                let entangled = bit_entangle(((x << 24), (y << 24) + (1 << 23)));
                entangle.push((entangled >> 48) as u16);
                detangle[(entangled >> 48) as usize] = (x as u8, y as u8);
                rotation.push(((entangled >> 44) & 0x0F) as u8);

                //  note to self: math is hard.
                //  rotation decode:    lsbs
                //  0100 -N--> 0100 --> 0100
                //  0100 -S--> 1000 --> 1110
                //  0100 -F--> 1011 --> 1100
                //  0100 -FS-> 0111 --> 0110
            }
        }

        return BytewiseHilbert {entangle: entangle, detangle: detangle, rotation: rotation};
    }

    pub fn entangle(&self, (mut x, mut y): (u32, u32)) -> u64 {
        let init_x = x;
        let init_y = y;
        let mut result = 0u64;
        for i in 0..4 {
            let x_byte = (x >> (24 - (8 * i))) as u8;
            let y_byte = (y >> (24 - (8 * i))) as u8;
            result = (result << 16) + self.entangle[(((x_byte as u16) << 8) + y_byte as u16) as usize] as u64;
            let rotation = self.rotation[(((x_byte as u16) << 8) + y_byte as u16) as usize];
            if (rotation & 0x2) > 0 { let temp = x; x = y; y = temp; }
            if rotation == 12 || rotation == 6 { x = 0xFFFFFFFF - x; y = 0xFFFFFFFF - y }
        }

        debug_assert!(bit_entangle((init_x, init_y)) == result);
        return result;
    }

    #[inline(always)]
    pub fn detangle(&self, tangle: u64) -> (u32, u32) {
        let init_tangle = tangle;
        let mut result = (0u32, 0u32);
        for log_s in 0u32..4 {
            let shifted = (tangle >> (16 * log_s)) as u16;
            let (x_byte, y_byte) = self.detangle[shifted as usize];
            let rotation = self.rotation[(((x_byte as u16) << 8) + y_byte as u16) as usize];
            if rotation == 12 || rotation == 6 {
                result.0 = (1 << 8 * log_s) - result.0 - 1;
                result.1 = (1 << 8 * log_s) - result.1 - 1;
            }
            if (rotation & 0x2) > 0 {
                let temp = result.0; result.0 = result.1; result.1 = temp;
            }

            result.0 += (x_byte as u32) << (8 * log_s);
            result.1 += (y_byte as u32) << (8 * log_s);
        }

        debug_assert!(bit_detangle(init_tangle) == result);
        return result;
    }
}

fn bit_entangle(mut pair: (u32, u32)) -> u64 {
    let mut result = 0u64;
    for log_s_rev in 0..32 {
        let log_s = 31 - log_s_rev;
        let rx = (pair.0 >> log_s) & 1u32;
        let ry = (pair.1 >> log_s) & 1u32;
        result += (((3 * rx) ^ ry) as u64) << (2 * log_s);
        pair = bit_rotate(log_s, pair, rx, ry);
    }

    return result;
}

fn bit_detangle(tangle: u64) -> (u32, u32) {
    let mut result = (0u32, 0u32);
    for log_s in 0..32 {
        let shifted = ((tangle >> (2 * log_s)) & 3u64) as u32;

        let rx = (shifted >> 1) & 1u32;
        let ry = (shifted ^ rx) & 1u32;
        result = bit_rotate(log_s, result, rx, ry);
        result = (result.0 + (rx << log_s), result.1 + (ry << log_s));
    }

    return result;
}

fn bit_rotate(logn: usize, pair: (u32, u32), rx: u32, ry: u32) -> (u32, u32) {
    if ry == 0 {
        if rx != 0 {
            ((1 << logn) - pair.1 - 1, (1 << logn) - pair.0 - 1)
        }
        else { (pair.1, pair.0) }
    }
    else { pair }
}
