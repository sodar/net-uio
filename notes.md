# Notes


- According to the spec 82540EM supports only the 32-bit PCI mode.
- `/sys/class/uio/uio0/device` is symlinked to `/sys/bus/pci/devices/0000:00:04.0`.
- `/sys/bus/pci/devices/0000:00:04.0/resource`:
    - contains addresses of PCI resources,
    - 1st cell - start address,
    - 2nd cell - end address,
    - 3rd cell - unknown
