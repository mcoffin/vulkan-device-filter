# vulkan-device-filter

A vulkan layer for filtering devices similar to the way that [`dxvk`](https://github.com/doitsujin/dxvk) does.

# Installation

To install as an implicit layer by copying the target files

```
cd vulkan-device-filter-layer
cargo build --release
./install.sh
```

To install as an implicit layer, you can use symlinks via the provided script.

```
cd vulkan-device-filter-layer
cargo build --release
./install-link.sh release
```

# Usage

There are two environment variables to be used to enable the layer, and subsequently filter the device names.

```bash
VK_DEVICE_FILTER_ENABLE=1 \
    VK_DEVICE_FILTER=$regex \
    $command
```
