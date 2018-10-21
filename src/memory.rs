extern crate byteorder;
extern crate libc;

use byteorder::{LittleEndian, ReadBytesExt};
use libc::*;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::io;
use std::mem;

#[derive(Debug)]
pub struct Memory {
    va: *mut c_void,
    size: usize,
    pa: Option<usize>,
}

impl Drop for Memory {
    fn drop(&mut self) {
        raw_deallocate_dma_memory(self.va, self.size);
    }
}

impl Memory {
    pub fn get_phys_addr(&mut self) -> usize {
        match self.pa {
            Some(addr) => addr,
            None => {
                let addr = find_phys_addr_for(self.va)
                    .expect("Failed to find physical address");
                self.pa = Some(addr);
                addr
            }
        }
    }
}

pub fn allocate_dma_memory(size: usize) -> Memory {
    let ptr = raw_allocate_dma_memory(size).expect("Failed to allocate memory for DMA");

    Memory {
        va: ptr,
        size: size,
        pa: None,
    }
}

fn raw_allocate_dma_memory(size: usize) -> Result<*mut c_void, io::Error> {
    assert_eq!(size % (2 * 1024 * 1024), 0, "`size` must be a multiple of 2 MB");

    let addr = 0 as *mut c_void;
    let length = size as size_t;
    let prot = PROT_READ | PROT_WRITE as c_int;
    let flags = MAP_PRIVATE | MAP_ANONYMOUS | MAP_HUGETLB as c_int;
    let fd = -1 as c_int;
    let offset = 0 as off_t;
    let ptr = unsafe { mmap(addr, length, prot, flags, fd, offset) };

    match ptr {
        MAP_FAILED => Err(io::Error::last_os_error()),
        _ => {
            // NOTE: Huge page is now reserved for us, but not allocated.
            // We have to provoke a page fault to force an allocation.
            let p = ptr as *mut u8;
            unsafe { *p = 0 };

            Ok(ptr)
        }
    }
}

fn raw_deallocate_dma_memory(ptr: *mut c_void, size: usize) {
    let ret = unsafe { munmap(ptr, size) };
    if ret == -1 {
        panic!("{:?}", io::Error::last_os_error());
    }
}

fn find_phys_addr_for(va: *mut c_void) -> Result<usize, io::Error> {
    let mut pagemap = File::open("/proc/self/pagemap").unwrap();
    let pagesize = 4096;
    let offset = (va as u64) / pagesize * (mem::size_of::<u64>() as u64);
    pagemap.seek(SeekFrom::Start(offset)).unwrap();

    let mut buf = [0u8; 8];
    pagemap.read(&mut buf).unwrap();
    let data: u64 = (&buf[..]).read_u64::<LittleEndian>().unwrap();

    let is_present = (data & (1 << 63)) >> 63;
    if is_present == 0 {
        panic!("Page is not present");
    }

    // NOTE: PFN is calculated simply as (phys_addr >> PAGE_SHIFT). By default, PAGE_SHIFT = 12.
    // NOTE: To check if address is mapped correctly - run pmemsave on `phys` address
    // inside q QEMU monitor.
    let phys = data << 12;

    Ok(phys as usize)
}
