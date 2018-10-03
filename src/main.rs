extern crate byteorder;
extern crate libc;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;
use std::fs::OpenOptions;

mod other;
mod resource;

use other::print_config_space;
use resource::Resource;

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

    let mut resource0 = Resource::new(BAR0_PATH).unwrap();
    println!("net-uio: resource0 = {:?}", resource0);

    let ctrl_reg = resource0.read_register(0);
    println!("net-uio: CTRL = {:#010x}", ctrl_reg);

    let ims_reg = resource0.read_register(0xd0);
    println!("net-uio: IMS = {:#010x}", ims_reg);

    resource0.write_register(0xd0, 0b100);
    println!("net-uio: IMS set LSC bit (link status change)");

    let icr_reg = resource0.read_register(0xc0);
    println!("net-uio: ICR = {:#010x}", icr_reg);

    let icr_reg = resource0.read_register(0xc0);
    println!("net-uio: ICR = {:#010x}", icr_reg);

    let mut buf = [0u8; 4];
    uio.read(&mut buf).unwrap();
    let interrupts = read_u32(&buf, 0x0);
    println!("net-uio: Interrupts = {}", interrupts);
}
