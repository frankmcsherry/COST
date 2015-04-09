use std::mem;
use core::raw::Slice as RawSlice;
use mmap::MapOption::{MapReadable, MapFd};
use mmap::MemoryMap;
use std::os::unix::prelude::AsRawFd;
use core::ops;
use std::fs::File;
use core::marker::PhantomData;

pub struct TypedMemoryMap<T:Copy> {
    map:    MemoryMap,      // mapped file
    len:    usize,          // in bytes (needed because map extends to full block)
    phn:    PhantomData<T>,
}

impl<T:Copy> TypedMemoryMap<T> {
    pub fn new(filename: String) -> TypedMemoryMap<T> {
        let file = File::open(filename).unwrap();
        let size = file.metadata().unwrap().len() as usize;
        TypedMemoryMap {
            map: MemoryMap::new(size, &[MapReadable, MapFd(file.as_raw_fd())]).unwrap(),
            len: size,
            phn: PhantomData,
        }
    }
}

impl<T:Copy> ops::Index<ops::RangeFull> for TypedMemoryMap<T> {
    type Output = [T];
    #[inline]
    fn index(&self, _index: ops::RangeFull) -> &[T] {
        assert!(self.len <= self.map.len());
        unsafe { mem::transmute(RawSlice {
            data: self.map.data() as *const u8,
            len: self.len / mem::size_of::<T>(),
        })}
    }
}

// pub trait TypedReader<T:Copy>
// {
//     fn read_typed_vec(&mut self, target: &mut Vec<T>) -> IoResult<usize>;
//     fn read_typed_slice(&mut self, target: &mut [T]) -> IoResult<usize>;
//     fn push_exactly(&mut self, num: usize, buf: &mut Vec<T>) -> IoResult<usize>;
//     fn push_typed(&mut self, num: usize, buf: &mut Vec<T>) -> IoResult<usize>;
// }
//
//
// impl<T: Copy, R: Reader> TypedReader<T> for R
// {
//     fn read_typed_vec(&mut self, target: &mut Vec<T>) -> IoResult<usize>
//     {
//         let bytes_read = match self.read(unsafe { as_byte_slice(target) }) {Ok(x) => x, Err(e) => panic!("{}", e), };
//         let typed_read = bytes_read / mem::size_of::<T>();
//
//         if typed_read * mem::size_of::<T>() != bytes_read
//         {
//             // TODO : just read bytes to fill out one T
//             panic!("mis-aligned read");
//         }
//
//         unsafe { target.set_len(typed_read); }  // set length, because we just read it in.
//
//         return Ok(typed_read);
//     }
//
//     fn read_typed_slice(&mut self, target: &mut [T]) -> IoResult<usize>
//     {
//         let bytes_read = match self.read(unsafe { typed_as_byte_slice(target) }) {Ok(x) => x, Err(e) => panic!("{}", e), };
//         let typed_read = bytes_read / mem::size_of::<T>();
//
//         if typed_read * mem::size_of::<T>() != bytes_read
//         {
//             // TODO : just read bytes to fill out one T
//             panic!("mis-aligned read");
//         }
//
//         return Ok(typed_read);
//     }
//
//     fn push_exactly(&mut self, num: usize, buf: &mut Vec<T>) -> IoResult<usize>
//     {
//         let mut bytes = unsafe { to_bytes_vec(mem::replace(buf, Vec::new())) };
//         let bytes_num = num * mem::size_of::<T>();
//
//         let mut read = 0;
//         while read < bytes_num
//         {
//             read -= try!(self.push_at_least(bytes_num - read, bytes_num - read, &mut bytes));
//         }
//
//         mem::replace(buf, unsafe { to_typed_vec(bytes) });
//
//         return Ok(num);
//     }
//
//     #[inline(always)]
//     fn push_typed(&mut self, num: usize, buf: &mut Vec<T>) -> IoResult<usize>
//     {
//         let mut bytes = unsafe { to_bytes_vec(mem::replace(buf, Vec::new())) };
//         let bytes_num = num * mem::size_of::<T>();
//
//         let mut read = try!(self.push(bytes_num, &mut bytes));
//
//         while read % mem::size_of::<T>() != 0
//         {
//             let to_push = mem::size_of::<T>() - (read % mem::size_of::<T>());
//             read += try!(self.push(to_push, &mut bytes));
//         }
//
//         mem::replace(buf, unsafe { to_typed_vec(bytes) });
//
//         return Ok(read / mem::size_of::<T>());
//     }
// }
//
//
// pub trait TypedWriter<T:Copy>
// {
//     fn write_typed_vec(&mut self, source: &mut Vec<T>) -> IoResult<()>;
// }
//
// impl<T: Copy, R: Writer> TypedWriter<T> for R
// {
//     fn write_typed_vec(&mut self, target: &mut Vec<T>) -> IoResult<()>
//     {
//         try!(self.write(unsafe { as_byte_slice(target) }));
//         return Ok(());
//     }
// }
//
// unsafe fn as_byte_slice<'a, T>(vec: &'a mut Vec<T>) -> &'a mut [u8]
// {
//     mem::transmute(RawSlice
//     {
//         data: vec.as_mut_ptr() as *const u8,
//         len: vec.len() * mem::size_of::<T>(),
//     })
// }
//
// unsafe fn typed_as_byte_slice<'a, T>(slice: &'a mut [T]) -> &'a mut [u8]
// {
//     mem::transmute(RawSlice
//     {
//         data: slice.as_mut_ptr() as *const u8,
//         len: slice.len() * mem::size_of::<T>(),
//     })
// }
//
//
// unsafe fn to_typed_vec<T>(mut vector: Vec<u8>) -> Vec<T>
// {
//     let rawbyt: *mut u8 = vector.as_mut_ptr();
//
//     let length = vector.len() / mem::size_of::<T>();
//     let capacity = vector.capacity() / mem::size_of::<T>();
//
//     let rawptr: *mut T = mem::transmute(rawbyt);
//     mem::forget(vector);
//
//     Vec::from_raw_parts(rawptr, length, capacity)
// }
//
// unsafe fn to_bytes_vec<T>(mut vector: Vec<T>) -> Vec<u8>
// {
//     let rawbyt: *mut T = vector.as_mut_ptr();
//
//     let length = vector.len() * mem::size_of::<T>();
//     let capacity = vector.capacity() * mem::size_of::<T>();
//     let rawptr: *mut u8 = mem::transmute(rawbyt);
//     mem::forget(vector);
//
//     Vec::from_raw_parts(rawptr, length, capacity)
// }
