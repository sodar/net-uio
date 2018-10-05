extern crate byteorder;
extern crate libc;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, SeekFrom};
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

    // Before waiting for any interrupt, `Interrupt Disable` bit of PCI command
    // register must be cleared. Interrupts are triggered if and only if
    // `Interrupt Disable` is cleared and appropriate bit is set in IMS register.
    let command_enable = !0x0400;
    cfg.seek(SeekFrom::Start(4)).unwrap();
    cfg.write_u16::<LittleEndian>(command_enable).unwrap();

    let status_reg = read_u16(&buffer, 6);
    println!("net-uio: status_reg  = {:#06x}", status_reg);

    let bar_reg_low = read_u32(&buffer, 0x10);
    let bar_reg_high = read_u32(&buffer, 0x14);

    println!("net-uio: bar_reg, low = {:#010x}, high = {:#010x}",
        bar_reg_low, bar_reg_high);

    let mut resource0 = Resource::new(BAR0_PATH).unwrap();
    println!("net-uio: resource0 = {:?}", resource0);

    // Read CTRL - control register.
    let ctrl_reg = resource0.read_register(0);
    println!("net-uio: CTRL = {:#010x}", ctrl_reg);

    // Read STATUS - device status register.
    let status_reg = resource0.read_register(0x8);
    println!("net-uio: STATUS = {:#010x}", status_reg);

    // Read IMS - interrupt mask set/read register.
    let ims_reg = resource0.read_register(0xd0);
    println!("net-uio: IMS = {:#010x}", ims_reg);

    // Enable interrupts for list status changes.
    resource0.write_register(0xd0, 0b100);
    println!("net-uio: IMS set LSC bit (link status change)");

    // Wait for any interrupt.
    let mut buf = [0u8; 4];
    uio.read(&mut buf).unwrap();
    let interrupts = read_u32(&buf, 0x0);
    println!("net-uio: Interrupts = {}", interrupts);

    // Read ICR - interrupt cause read register. Reading it acknowledges any pending
    // interrupt events, thus reads that follow will read 0 on particular bits.
    let icr_reg = resource0.read_register(0xc0);
    println!("net-uio: ICR = {:#010x}", icr_reg);

    let icr_reg = resource0.read_register(0xc0);
    println!("net-uio: ICR = {:#010x}", icr_reg);
}
