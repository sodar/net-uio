extern crate byteorder;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;
use std::fs::OpenOptions;

static DEV_PATH: &str = "/dev/uio0";
static CFG_PATH: &str = "/sys/class/uio/uio0/device/config";

fn read_u16(buf: &[u8], off: usize) -> u16 {
    (&buf[off .. off + 2]).read_u16::<LittleEndian>().unwrap()
}

fn read_u32(buf: &[u8], off: usize) -> u32 {
    (&buf[off .. off + 4]).read_u32::<LittleEndian>().unwrap()
}

fn print_config_space(buf: &[u8; 256]) {
    for i in 0..32 {
        let ii = i * 4;
        print!("net-uio: cfg[{:02x}]:", ii);
        for j in 0..4 {
            let jj = ii + j;
            print!(" {:02x}", buf[jj]);
        }
        print!("\n");
    }
}

fn main() {
    let mut opts = OpenOptions::new();
    let _uio = opts.read(true).write(true).open(DEV_PATH).unwrap();
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
}
