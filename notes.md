# Notes


- According to the spec 82540EM supports only the 32-bit PCI mode.
- `/sys/class/uio/uio0/device` is symlinked to `/sys/bus/pci/devices/0000:00:04.0`.
- `/sys/bus/pci/devices/0000:00:04.0/resource`:
    - contains addresses of PCI resources,
    - 1st cell - start address,
    - 2nd cell - end address,
    - 3rd cell - unknown


## QEMU

- QEMU monitor:
    - To enter: `Ctrl + Alt + 2`
    - To exit: `Ctrl + Alt + 1`
- Show network info:

    ```
    (qemu) info network
    virtio-net-pci.0: index=0,type=nic,model=virtio-net-pci,macaddr=52:54:00:12:34:56
     \ net0: index=0,type=user,net=10.0.2.0,restrict=off
    e1000.0: index=0,type=nic,model=e1000,macaddr=52:54:83:58:6b:df
     \ net1: index=0,type=tap,ifname=tap0,script=/etc/qemu-ifup,downscript=/etc/qemu-ifdown
    ```


## Testing

- Link status:

    ```
    # You can emulate plugging/unplugging of the network cable using set_link
    # command in QEMU monitor
    (qemu) set_link net1 on
    # LSC interrupt should be generated
    # bit 1 in STATUS should be equal to 1 - link is UP
    (qemu) set_link net1 off
    # LSC interrupt should be generated
    # bit 1 in STATUS should be equal to 0 - link is DOWN
    ```


## Interrupt handling

Summary of the interrupt handling:

- Set to `1` appropriate interrupt bits in `IMS` register of NIC register map.
    - Example: Bit 2 of `IMS` is `Link Status Change` interrupt, triggered every time link status changes from up to down, or otherwise.
- Read `ICR` NIC register to clear it.
    - `ICR` register holds information about what interrupt causing events occurred.
      Interrupt line is asserted if and only if appropriate `ICR` bit is zeroed and appropriate bit of `IMS` is set to `1`.
- Reenable interrupts by writing `0` to `Interrupt disable` bit in PCI command register.
- Read from UIO device file to wait for any interrupts.
    - After interrupt has been triggered, `uio_pci_generic` disables interrupts.
- Software has to acknowledge the interrupts before reenabling them.
  Without aknowledging, those interrupts will be triggered again.
  Interrupts can be acknowledged by reading from `ICR` register.


## Huge pages

Here are some quick notes regarding huge pages.

```bash
# Show huge page statistics.
cat /proc/meminfo | grep Huge

# Allocate huge pages at runtime.
echo 16 > /proc/sys/vm/nr_hugepages

# Mount hugetlbfs for mmaping.
mount -t hugetlbfs -o size=100%,min_size=100%,nr_inodes=16 none /mnt/huge

# After mounting, in /proc/meminfo, `HugePages_Rsvd` should be equal to `HugePages_Total`.
# This means, that whole huge pool has been reserved for allocation and the following allocations
# will succeed.

# IMPORTANT: Stopped using this method, since I could find quickly how to create a proper file.
#            Unmoount hugetlbfs to proceed.

# Map huge page using anonymous mapping by providing:
# - FLAGS = PRIVATE | ANONYMOUS | HUGETLB

# After mapping, in /proc/meminfo, `HugePages_Free` should reduce by a number of mapped pages.
```
