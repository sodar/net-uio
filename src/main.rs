use std::io::Read;
use std::fs::OpenOptions;

static DEV_PATH: &str = "/dev/uio0";
static CFG_PATH: &str = "/sys/class/uio/uio0/device/config";

fn main() {
    let mut opts = OpenOptions::new();
    let _uio = opts.read(true).write(true).open(DEV_PATH).unwrap();

    let mut opts = OpenOptions::new();
    let mut cfg = opts.read(true).write(true).open(CFG_PATH).unwrap();

    let mut buffer = [0u8; 256];
    let bytes = cfg.read(&mut buffer).unwrap();
    println!("[net-uio] Read {} bytes from uio0 config space", bytes);

    for i in 0..32 {
        let ii = i * 8;
        print!("[net-uio] cfg[{:02x}]:", ii);
        for j in 0..8 {
            let jj = ii + j;
            print!(" {:02x}", buffer[jj]);
        }
        print!("\n");
    }
}
