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
