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
}
