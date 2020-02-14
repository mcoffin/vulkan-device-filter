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

You'll also need the dynamically linked rust stdlib somewhere findable.

```bash
export LD_LIBRARY_PATH=$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib

VK_INSTANCE_LAYERS=VK_LAYER_MCOF_device_filter \
    VK_DEVICE_FILTER=$regex \
    $command
```

## Config file

The layer can also source it's filtering information from a config file in these locations (first = higher precedence for matching)

1. `${XDG_CONFIG_HOME:-~/.config}/vulkan-device-filter/config.yml`
2. `/etc/vulkan-device-filter/config.yml`
3. `/usr/share/vulkan-device-filter/config.yml`

The config file path may also be overridden with the environment variable `VK_DEVICE_FILTER_CONFIG`.

The following example config file forces the use of a GTX 1650 for the `mpv` application, by matching on either the executable path, or the [VkApplicationInfo](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkApplicationInfo.html) struct submitted by the application.

```yaml
filters:
  - filter: '1650'
    match_rule:
      Or:
        rules:
          - Executable:
              name: /mpv$
          - AppInfo:
              name: mpv
```
