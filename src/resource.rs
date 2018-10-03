extern crate libc;

use std::io;
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;

use libc::*;

#[derive(Debug)]
pub struct Resource {
    file: File,
    ptr: *mut u8,
    size: usize,
}

impl Resource {
    pub fn new(resource_path: &str) -> Result<Resource, io::Error> {
        let mut opts = OpenOptions::new();
        let file = opts.read(true).write(true).open(resource_path)?;

        let (map, size) = map_file(&file);

        let resource = Resource {
            file: file,
            ptr: map,
            size: size,
        };

        Ok(resource)
    }

    pub fn read_register(&self, offset: isize) -> u32 {
        unsafe {
            let p = self.ptr.offset(offset);
            let p = p as *mut u32;
            *p
        }
    }

    pub fn write_register(&mut self, offset: isize, value: u32) {
        unsafe {
            let p = self.ptr.offset(offset);
            let p = p as *mut u32;
            *p = value;
        }
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        let ptr = self.ptr as *mut c_void;
        let ret = unsafe { munmap(ptr, self.size) };
        assert_eq!(ret, 0);
    }
}

fn map_file(file: &File) -> (*mut u8, usize) {
    let size: usize = 131072;
    let fd = file.as_raw_fd();

    let addr: *mut c_void = 0 as *mut c_void;
    let len: size_t = size;
    let prot = PROT_READ | PROT_WRITE;
    let flags = MAP_SHARED;
    // let fd
    let offset: off_t = 0;

    let p: *mut c_void = unsafe {
        mmap(addr, len, prot, flags, fd, offset)
    };
    let p = p as *mut u8;

    (p, size)
}
