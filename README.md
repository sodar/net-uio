# net-uio

Networking driver utilizing Linux UIO subsystem, written for fun.


## Binding device

Requirements:

- QEMU virtual machine with `e1000` NIC attached
    - `e1000` emulates Intel 82540EM Gigabit Ethernet NIC
    - this NIC has PCI device ID `8086:100e`

To bind the device to UIO driver:

```bash
modprobe uio_pci_generic
echo "8086 100e" > /sys/bus/pci/drivers/uio_pci_generic/new_id

echo -n '0000:00:04.0' > /sys/bus/pci/drivers/e1000/unbind
echo -n '0000:00:04.0' > /sys/bus/pci/drivers/uio_pci_generic/bind

ls -l /sys/bus/pci/devices/0000:00:04.0/driver
```


## References

- OSDev Wiki Entry about Intel 8254x card family:
  [https://wiki.osdev.org/Intel_8254x](https://wiki.osdev.org/Intel_8254x)
- Intel 8254x Software Developer's Manual:
  [https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf](https://www.intel.com/content/dam/doc/manual/pci-pci-x-family-gbe-controllers-software-dev-manual.pdf)
