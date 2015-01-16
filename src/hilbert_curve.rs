use graph_iterator::EdgeMapper;

pub fn convert_to_hilbert<I, O>(graph: &I, make_dense: bool, mut output: O) -> ()
where I : EdgeMapper,
      O : FnMut(u16, u16, u32, &Vec<(u16, u16)>) -> (),
{
    let mut uppers = Vec::new();
    let mut names = Vec::new();
    let mut names_count = 0i32;
    let hilbert = BytewiseHilbert::new();

    graph.map_edges(|mut node, mut edge| {
        if make_dense {
            while names.len() as u32 <= node { names.push(-1i32); }
            while names.len() as u32 <= edge { names.push(-1i32); }
            if names[node as usize] == -1i32 { names[node as usize] = names_count; node = names_count as u32; names_count += 1; }
            if names[edge as usize] == -1i32 { names[edge as usize] = names_count; edge = names_count as u32; names_count += 1; }
        }

        let entangled = hilbert.entangle((node as u32, edge as u32));
        let upper = (entangled >> 32) as u32;
        let lower = entangled as u32;

        while uppers.len() as u32 <= upper { uppers.push(Vec::new()); }
        uppers[upper as usize].push(lower);
    });

    let mut temp = Vec::new();
    for (upper, lowers) in uppers.iter_mut().enumerate() {
        if lowers.len() > 0 {
            let upper = upper as u64;
            let upair = hilbert.detangle((upper as u64) << 32);
            let upperx = (upair.0 >> 16) as u16;
            let uppery = (upair.1 >> 16) as u16;
            let length = lowers.len() as u32;

            lowers.sort();  // TODO : Check Radix sort perf
            temp.clear();

            for &lower in lowers.iter() {
                // TODO : Could cache mirror/flip behavior from upper rather than recompute.
                let lpair = hilbert.detangle(upper + (lower as u64));
                let lowerx = (lpair.0 & 65535u32) as u16;
                let lowery = (lpair.1 & 65535u32) as u16;
                temp.push((lowerx, lowery));
            }

            output(upperx, uppery, length, &temp);
        }
    }
}

// algorithm drawn in large part from http://en.wikipedia.org/wiki/Hilbert_curve
// bytewise implementation based on tracking cumulative rotation / mirroring.

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
        for x in (0u32..256) {
            for y in (0u32..256) {
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
        for i in (0us .. 4) {
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

    pub fn detangle(&self, tangle: u64) -> (u32, u32) {
        let init_tangle = tangle;
        let mut result = (0u32, 0u32);
        for log_s in (0u32 .. 4) {
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

// pub struct BitwiseHilbert;
//
// impl BitwiseHilbert {
//     pub fn entangle(&self, (x, y): (u32, u32)) -> u64 { bit_entangle((x, y)) }
//     pub fn detangle(&self, tangle: u64) -> (u32, u32) { bit_detangle(tangle) }
// }

fn bit_entangle(mut pair: (u32, u32)) -> u64 {
    let mut result = 0u64;
    for log_s_rev in (0us .. 32us) {
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
    for log_s in (0us .. 32us) {
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
