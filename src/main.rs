extern crate byteorder;
extern crate libc;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

mod device;
mod resource;

use device::UioPciDevice;
use resource::Resource;

static DEV_PATH: &str = "/dev/uio0";
static CFG_PATH: &str = "/sys/class/uio/uio0/device/config";
static BAR0_PATH: &str = "/sys/class/uio/uio0/device/resource0";

fn read_u16(buf: &[u8], off: usize) -> u16 {
    (&buf[off .. off + 2]).read_u16::<LittleEndian>().unwrap()
}

fn main() {
    let mut device = UioPciDevice::new(DEV_PATH, CFG_PATH);

    let mut buffer = [0u8; 256];
    let bytes = device.cfg.read(&mut buffer).unwrap();
    println!("net-uio: Read {} bytes from {}", bytes, CFG_PATH);

    let vendor_id = read_u16(&buffer, 0);
    println!("net-uio: vendor_id   = {:#06x}", vendor_id);

    let device_id = read_u16(&buffer, 2);
    println!("net-uio: device_id   = {:#06x}", device_id);

    let command_reg = read_u16(&buffer, 4);
    println!("net-uio: command_reg = {:#06x}", command_reg);

    let status_reg = read_u16(&buffer, 6);
    println!("net-uio: status_reg  = {:#06x}", status_reg);

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

    loop {
        device.reenable_interrupts();

        let interrupts = device.wait_for_interrupts();
        println!("net-uio: interrupt counter = {}", interrupts);

        let icr_reg = resource0.read_register(0xc0);
        println!("net-uio: ICR = {:#010x}", icr_reg);
    }
}
