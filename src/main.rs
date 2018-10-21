extern crate byteorder;
extern crate libc;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

mod device;
mod memory;
mod resource;

use device::UioPciDevice;
use resource::Resource;

static DEV_PATH: &str = "/dev/uio0";
static CFG_PATH: &str = "/sys/class/uio/uio0/device/config";
static BAR0_PATH: &str = "/sys/class/uio/uio0/device/resource0";

//
// Intel 8254x register offsets.
//

static CTRL  : isize = 0x0;
static STATUS: isize = 0x8;

// Interrupt registers.
static ICR: isize = 0x00c0;
static ITR: isize = 0x00c4;
#[allow(dead_code)]
static ICS: isize = 0x00c8;
static IMS: isize = 0x00d0;
#[allow(dead_code)]
static IMC: isize = 0x00d8;

// Receive descriptor registers.
static RCTL : isize = 0x0100;
static RDBAL: isize = 0x2800;
static RDBAH: isize = 0x2804;
static RDLEN: isize = 0x2808;
static RDH  : isize = 0x2810;
static RDT  : isize = 0x2818;
static RDTR : isize = 0x2820;

fn read_u16(buf: &[u8], off: usize) -> u16 {
    (&buf[off .. off + 2]).read_u16::<LittleEndian>().unwrap()
}

fn load_and_print_pci_config(device: &mut UioPciDevice) {
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
}

fn main() {
    let mut device = UioPciDevice::new(DEV_PATH, CFG_PATH);
    load_and_print_pci_config(&mut device);

    let mut resource0 = Resource::new(BAR0_PATH).unwrap();
    println!("net-uio: resource0 = {:?}", resource0);

    // Read CTRL - control register.
    let ctrl_reg = resource0.read_register(CTRL);
    println!("net-uio: CTRL = {:#010x}", ctrl_reg);

    // Read STATUS - device status register.
    let status_reg = resource0.read_register(STATUS);
    println!("net-uio: STATUS = {:#010x}", status_reg);

    // Read IMS - interrupt mask set/read register.
    let ims_reg = resource0.read_register(IMS);
    println!("net-uio: IMS = {:#010x}", ims_reg);

    println!("net-uio: ITR = {:#010x}", resource0.read_register(ITR));

    resource0.write_register(IMS, 0b100);

    // Print out receiver registers.
    println!("net-uio: Receiver registers.");
    println!("net-uio: RCTL  = {:#010x}", resource0.read_register(RCTL));
    println!("net-uio: RDBAL = {:#010x}", resource0.read_register(RDBAL));
    println!("net-uio: RDBAH = {:#010x}", resource0.read_register(RDBAH));
    println!("net-uio: RDLEN = {:#010x}", resource0.read_register(RDLEN));
    println!("net-uio: RDH   = {:#010x}", resource0.read_register(RDH));
    println!("net-uio: RDT   = {:#010x}", resource0.read_register(RDT));
    println!("net-uio: RDTR  = {:#010x}", resource0.read_register(RDTR));

    // Enable interrupts for list status changes.
    resource0.write_register(IMS, 0b100);
    println!("net-uio: IMS set LSC bit (link status change)");

    let mut mem = memory::allocate_dma_memory(2 * 1024 * 1024);
    println!("net-uio: RX ring, mem = {:?}", mem);
    println!("net-uio: RX ring, pa = {:#x}", mem.get_phys_addr());

    loop {
        device.reenable_interrupts();

        let interrupts = device.wait_for_interrupts();
        println!("net-uio: interrupt counter = {}", interrupts);

        let icr_reg = resource0.read_register(ICR);
        println!("net-uio: ICR = {:#010x}", icr_reg);
    }
}
