pub fn print_config_space(buf: &[u8; 256]) {
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
