use std::mem;
use std::slice;
use std::ops;
use std::fs::File;
use std::marker::PhantomData;

use memmap;

pub struct TypedMemoryMap<T:Copy> {
    map: memmap::Mmap,
    len:    usize,              // in bytes (needed because map extends to full block)
    phn:    PhantomData<T>,
}

impl<T:Copy> TypedMemoryMap<T> {
    pub fn new(filename: String) -> TypedMemoryMap<T> {
        let file = File::open(filename).ok().expect("error opening file");
        let size = file.metadata().ok().expect("error reading metadata").len() as usize;

        TypedMemoryMap {
            map: memmap::Mmap::open(&file, memmap::Protection::Read).unwrap(),
            len: size / mem::size_of::<T>(),
            phn: PhantomData,
        }
    }
}

impl<T:Copy> ops::Index<ops::RangeFull> for TypedMemoryMap<T> {
    type Output = [T];
    #[inline]
    fn index(&self, _index: ops::RangeFull) -> &[T] {
        unsafe { slice::from_raw_parts(self.map.ptr() as *const T, self.len) }
    }
}
