extern crate byteorder;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, SeekFrom};
use std::fs::{File, OpenOptions};

pub struct UioPciDevice {
    // TODO: Should be private.
    pub uio: File,
    // TODO: Should be private.
    pub cfg: File,
}

impl UioPciDevice {
    pub fn new(dev_path: &str, cfg_path: &str) -> UioPciDevice {
        let open_file_with_rw = |path| {
            let mut opts = OpenOptions::new();
            // TODO: Do something with this unwrap().
            opts.read(true).write(true).open(path).unwrap()
        };

        UioPciDevice {
            uio: open_file_with_rw(dev_path),
            cfg: open_file_with_rw(cfg_path),
        }
    }

    // Before waiting for any interrupt, `Interrupt Disable` bit of PCI command
    // register must be cleared. Interrupts are triggered if and only if
    // `Interrupt Disable` is cleared and appropriate bit is set in IMS register.
    //
    // TODO: Should return a proper error.
    pub fn reenable_interrupts(&mut self) {
        let command_enable: u16 = !0x0400;
        self.cfg.seek(SeekFrom::Start(4))
            .expect("Failed to set a cursor on PCI command register");
        self.cfg.write_u16::<LittleEndian>(command_enable)
            .expect("Failed to update the PCI command register");
    }

    // TODO: Should return a proper error.
    pub fn wait_for_interrupts(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        self.uio.read(&mut buf)
            .expect("Failed reading from UIO device file");
        let interrupts = (&buf[..]).read_u32::<LittleEndian>()
            .expect("Failed interpreting UIO device output as u32");

        interrupts
    }
}
