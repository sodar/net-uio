extern crate byteorder;
extern crate libc;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;
use std::mem;
use std::slice;

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

// Receiver Control Register
static RCTL_EN: u32  = (1 << 1);
static RCTL_UPE: u32 = (1 << 3);
static RCTL_MPE: u32 = (1 << 4);

#[derive(Debug)]
#[repr(C)]
struct RxDescriptor {
    buffer_addr: u64,
    length: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    special: u16,
}

#[derive(Debug)]
struct RxBuffer {
    pub va: *mut u8,
    pub pa: usize,
}

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

    // Enable interrupts for list status changes.
    resource0.write_register(IMS, 0b100);
    println!("net-uio: IMS set LSC bit (link status change)");

    //
    // Allocate memory for RX descriptors.
    //
    let mut rx_desc_mem = memory::allocate_dma_memory(2 * 1024 * 1024);
    {
        let pa = rx_desc_mem.get_phys_addr();
        println!("net-uio: rx_desc_mem.pa = {:#x}", pa);
        println!("net-uio: rx_desc_mem = {:?}", rx_desc_mem);
    }
    let rx_desc_ptr_base = rx_desc_mem.va as *mut RxDescriptor;
    let rx_descriptors = unsafe { slice::from_raw_parts_mut(rx_desc_ptr_base, 8) };

    //
    // Allocate memory for packet buffers.
    //
    let mut rx_buffer_mem = memory::allocate_dma_memory(2 * 1024 * 1024);
    {
        let pa = rx_buffer_mem.get_phys_addr();
        println!("net-uio: rx_buffer_mem.pa = {:#x}", pa);
        println!("net-uio: rx_buffer_mem = {:?}", rx_buffer_mem);
    }

    let mut rx_buffers: Vec<RxBuffer> = Vec::new();
    for i in 0..8 {
        let va = rx_buffer_mem.va as *mut u8;
        let va = unsafe { va.offset(i * 2048) };

        let off = (i as usize) * 2048;
        let pa = rx_buffer_mem.get_phys_addr() + off;

        let buf = RxBuffer{ va: va, pa: pa };
        {
            let d = &mut rx_descriptors[i as usize];
            d.buffer_addr = buf.pa as u64;
        }
        rx_buffers.push(buf);
    }

    //
    // Configure Receiver.
    //
    {
        resource0.write_register(RCTL, 0);

        let rctl = RCTL_UPE | RCTL_MPE;
        resource0.write_register(RCTL, rctl);
        // Receiver configured for 2048 bytes packet buffers

        let pa = rx_desc_mem.get_phys_addr();
        let rdbah: u32 = ((pa & 0xffffffff00000000) >> 32) as u32;
        let rdbal: u32 = ((pa & 0x00000000ffffffff)      ) as u32;
        let rdlen: u32 = 8 * mem::size_of::<RxDescriptor>() as u32;
        let rdh: u32 = 0;
        let rdt: u32 = 1;

        resource0.write_register(RDBAH, rdbah);
        resource0.write_register(RDBAL, rdbal);
        resource0.write_register(RDLEN, rdlen);
        resource0.write_register(RDH, rdh);
        resource0.write_register(RDT, rdt);

        let rdtr: u32 = 0;
        resource0.write_register(RDTR, rdtr);

        // Enable receive timer interrupt.
        let ims: u32 = 1 << 7;
        resource0.write_register(IMS, ims);

        // Enable receiver
        let rctl = rctl | RCTL_EN;
        resource0.write_register(RCTL, rctl);
    }

    // Print out receiver registers.
    println!("net-uio: Receiver registers.");
    println!("net-uio: RCTL  = {:#010x}", resource0.read_register(RCTL));
    println!("net-uio: RDBAH = {:#010x}", resource0.read_register(RDBAH));
    println!("net-uio: RDBAL = {:#010x}", resource0.read_register(RDBAL));
    println!("net-uio: RDLEN = {:#010x}", resource0.read_register(RDLEN));
    println!("net-uio: RDH   = {:#010x}", resource0.read_register(RDH));
    println!("net-uio: RDT   = {:#010x}", resource0.read_register(RDT));
    println!("net-uio: RDTR  = {:#010x}", resource0.read_register(RDTR));

    loop {
        device.reenable_interrupts();

        let interrupts = device.wait_for_interrupts();
        println!("net-uio: interrupt counter = {}", interrupts);

        let icr_reg = resource0.read_register(ICR);
        println!("net-uio: ICR = {:#010x}", icr_reg);

        println!("net-uio: Receiver registers.");
        println!("net-uio: RCTL  = {:#010x}", resource0.read_register(RCTL));
        println!("net-uio: RDBAH = {:#010x}", resource0.read_register(RDBAH));
        println!("net-uio: RDBAL = {:#010x}", resource0.read_register(RDBAL));
        println!("net-uio: RDLEN = {:#010x}", resource0.read_register(RDLEN));
        println!("net-uio: RDH   = {:#010x}", resource0.read_register(RDH));
        println!("net-uio: RDT   = {:#010x}", resource0.read_register(RDT));
        println!("net-uio: RDTR  = {:#010x}", resource0.read_register(RDTR));
    }
}
