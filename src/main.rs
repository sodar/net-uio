extern crate byteorder;
extern crate libc;

use byteorder::{LittleEndian, ReadBytesExt};
use libc::*;
use std::io::Read;
use std::fs::OpenOptions;
use std::os::unix::io::IntoRawFd;

mod other;

use other::print_config_space;

static DEV_PATH: &str = "/dev/uio0";
static CFG_PATH: &str = "/sys/class/uio/uio0/device/config";
static BAR0_PATH: &str = "/sys/class/uio/uio0/device/resource0";

fn read_u16(buf: &[u8], off: usize) -> u16 {
    (&buf[off .. off + 2]).read_u16::<LittleEndian>().unwrap()
}

fn read_u32(buf: &[u8], off: usize) -> u32 {
    (&buf[off .. off + 4]).read_u32::<LittleEndian>().unwrap()
}

fn main() {
    let mut opts = OpenOptions::new();
    let mut uio = opts.read(true).write(true).open(DEV_PATH).unwrap();
    println!("net-uio: Opened {}", DEV_PATH);

    let mut opts = OpenOptions::new();
    let mut cfg = opts.read(true).write(true).open(CFG_PATH).unwrap();
    println!("net-uio: Opened {}", CFG_PATH);

    let mut buffer = [0u8; 256];
    let bytes = cfg.read(&mut buffer).unwrap();
    println!("net-uio: Read {} bytes from {}", bytes, CFG_PATH);

    print_config_space(&buffer);

    let vendor_id = read_u16(&buffer, 0);
    println!("net-uio: vendor_id   = {:#06x}", vendor_id);

    let device_id = read_u16(&buffer, 2);
    println!("net-uio: device_id   = {:#06x}", device_id);

    let command_reg = read_u16(&buffer, 4);
    println!("net-uio: command_reg = {:#06x}", command_reg);

    let status_reg = read_u16(&buffer, 6);
    println!("net-uio: status_reg  = {:#06x}", status_reg);

    let bar_reg_low = read_u32(&buffer, 0x10);
    let bar_reg_high = read_u32(&buffer, 0x14);

    println!("net-uio: bar_reg, low = {:#010x}, high = {:#010x}",
        bar_reg_low, bar_reg_high);

    let bar0 = OpenOptions::new().read(true).write(true)
        .open(BAR0_PATH)
        .expect("Unable to open PCI resource file");
    let bar0_fd = bar0.into_raw_fd();

    let addr: *mut c_void = 0 as *mut c_void;
    let len: size_t = 131072;
    let prot = PROT_READ | PROT_WRITE;
    let flags = MAP_SHARED;
    let fd: c_int = bar0_fd;
    let offset: off_t = 0;

    let m: *mut c_void;
    unsafe {
        m = mmap(addr, len, prot, flags, fd, offset)
    }
    println!("net-uio: mmap result = {:?}", m);

    let bar0_ptr = m as *mut u32;

    unsafe {
        let ctrl_reg = bar0_ptr.offset(0);
        println!("net-uio: CTRL pointer = {:?}", ctrl_reg);
        println!("net-uio: CTRL = {:#010x}", *ctrl_reg);
    }

    unsafe {
        let status_reg = bar0_ptr.offset(2);
        println!("net-uio: STATUS pointer = {:?}", status_reg);
        println!("net-uio: STATUS = {:#010x}", *status_reg);
    }

    // NOTE: Manipulating Interrupt Mask Set/Read register
    unsafe {
        let ims_reg = bar0_ptr.offset(52);
        println!("net-uio: IMS pointer = {:?}", ims_reg);
        println!("net-uio: IMS = {:#010x}", *ims_reg);

        let ims_reg_value: u32 = 0b100;
        *ims_reg = ims_reg_value;
        println!("net-uio: IMS setting LSC bit (link status change)");
        println!("net-uio: IMS = {:#010x}", *ims_reg);
    }

    // NOTE: Reading Interrupt Mask Cause Read Register
    unsafe {
        let icr_reg = bar0_ptr.offset(48);
        println!("net-uio: ICR pointer = {:?}", icr_reg);
        println!("net-uio: ICR = {:#010x}", *icr_reg);
        println!("net-uio: ICR = {:#010x}", *icr_reg);

        //let icr_reg_value: u32 = 0b100;
        //*icr_reg = icr_reg_value;
        //println!("net-uio: ICR, ");
    }

    let mut buffer = [0u8; 4];
    assert_eq!(uio.read(&mut buffer).unwrap(), 4);

    unsafe {
        assert_eq!(munmap(m, 131072), 0);
    }

    unsafe { assert_eq!(close(bar0_fd), 0); }
}
