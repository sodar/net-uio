extern crate byteorder;
extern crate libc;

use byteorder::{LittleEndian, ReadBytesExt};
use libc::*;
use std::fs::File;
use std::io::{BufReader, BufRead, Error, Read, Seek, SeekFrom};
use std::mem;

fn main() {
    let ret = unsafe { mlockall(MCL_CURRENT | MCL_FUTURE) };
    if ret == -1 {
        let err = Error::from_raw_os_error(ret as i32);
        panic!("Error on locking pages: {:?}", err);
    }
    println!("physmap: Successfully locked all pages (current and future)");

    let addr = 0 as *mut c_void;
    let len: size_t = 4096;
    let prot = PROT_READ | PROT_WRITE;
    let flags = MAP_ANONYMOUS | MAP_PRIVATE;
    let fd: c_int = -1;
    let off: off_t = 0;

    let buf = unsafe { mmap(addr, len, prot, flags, fd, off) };
    if buf == MAP_FAILED {
        panic!("Failed to define an anonymous mapping");
    }

    {
        // Write some test data - verify them using pmemsave in QEMU monitor.
        let ptr = buf as *mut u32;
        unsafe {
            *ptr = 0x12345678;
        }
    }

    println!("physmap: buf = {:?}", buf);

    // Code below is kind of not needed since I already know the virtual address
    // which I want. But I did it anyway.
    let maps = File::open("/proc/self/maps").unwrap();
    let mut reader = BufReader::new(maps);
    loop {
        let mut line = String::new();
        let len = reader.read_line(&mut line).unwrap();
        if len == 0 {
            break;
        }
        let v: Vec<&str> = line.split(' ').collect();
        let addr: Vec<&str> = v[0].split('-').collect();
        let start = u64::from_str_radix(addr[0], 16).unwrap();
        let _end = u64::from_str_radix(addr[1], 16).unwrap();

        let my_addr = buf as u64;
        if my_addr == start {
            println!("physmap: Found anonymous mapping in /proc/self/maps");
            println!("physmap: {}", line);
        }
    }

    println!("physmap: Analyzing /proc/self/pagemap");
    let mut pagemap = File::open("/proc/self/pagemap").unwrap();
    let pagesize = 4096;
    let offset = (buf as u64) / pagesize * (mem::size_of::<u64>() as u64);
    pagemap.seek(SeekFrom::Start(offset)).unwrap();
    {
        let mut buf = [0u8; 8];
        pagemap.read(&mut buf).unwrap();
        let data: u64 = (&buf[..]).read_u64::<LittleEndian>().unwrap();
        println!("physmap: page.data = {:#x}", data);
        println!("physmap: page.data = {:#b}", data);

        // NOTE: Theoretically << 9 is correct, because PFN is 0:54.
        // It appears that 3 most significant bits are zeroed always??
        // NOTE: To check if address is mapped correctly - run pmemsave on `phys` address
        // inside q QEMU monitor.
        let phys = data << 12;
        println!("physmap: page.phys = {:#x}", phys);
        println!("physmap: page.phys = {:#b}", phys);

        let is_present = (data & (1 << 63)) >> 63;
        println!("physmap: page.is_present = {}", is_present);
    }

    loop {
        // nothing
    }
}
