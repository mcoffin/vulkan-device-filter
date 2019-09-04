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

## Manual installation

If you don't want to use the provided install script(s), you can move the target files manually.

```bash
cd vulkan-device-filter-layer
cargo build --release
install -D -m755 ../target/release/libvulkan_device_filter_layer.so $target_dir/libVkLayer_device_filter.so
install -D -m644 -t $target_dir VkLayer_device_filter.json
```

# Usage

There are two environment variables to be used to enable the layer, and subsequently filter the device names.

```bash
LD_LIBRARY_PATH=$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib \
    VK_DEVICE_FILTER_ENABLE=1 \
    VK_DEVICE_FILTER=$regex \
    $command
```
