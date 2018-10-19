#!/bin/bash
set -e
set -x

# This script needs to be run as root.

modprobe uio_pci_generic

echo '8086 100e' > /sys/bus/pci/drivers/uio_pci_generic/new_id

echo -n '0000:00:04.0' > /sys/bus/pci/drivers/e1000/unbind
echo -n '0000:00:04.0' > /sys/bus/pci/drivers/uio_pci_generic/bind
