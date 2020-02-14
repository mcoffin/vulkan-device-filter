# vulkan-device-filter

A vulkan layer for filtering devices similar to the way that [`dxvk`](https://github.com/doitsujin/dxvk) does.

# Usage

There are two environment variables to be used to enable the layer, and subsequently filter the device names.

You'll also need the dynamically linked rust stdlib somewhere findable.

To enable the layer, set the following two environment variables.

```bash
export LD_LIBRARY_PATH=$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib
export VK_INSTANCE_LAYERS=VK_LAYER_MCOF_device_filter
```

## Via Config file (recommended)

The layer can also source it's filtering information from a config file in these locations (first = higher precedence for matching)

1. `${XDG_CONFIG_HOME:-~/.config}/vulkan-device-filter/config.yml`
2. `/etc/vulkan-device-filter/config.yml`
3. `/usr/share/vulkan-device-filter/config.yml`

The config file path may also be overridden with the environment variable `VK_DEVICE_FILTER_CONFIG`.

The following example config file forces the use of a GTX 1650 for the the game The Talos Principle, and the use of the intel iGPU for [mpv](https://mpv.io/) by matching on either the executable path, or the [VkApplicationInfo](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkApplicationInfo.html) struct submitted by the application.

Full information for the structure of this file can be cleaned from the structs in [the `config` module](https://gitlab.com/mcoffin/vulkan-device-filter/tree/master/vulkan-device-filter-layer/src/config/mod.rs).

```yaml
filters:
  - filter: '1650'
    match_rule:
      Executable:
        name: Talos
  - filter: 'Intel'
    match_rule:
      Or:
        rules:
          - Executable:
              name: ^/usr/(local/)?bin/mpv$
          - AppInfo:
              name: mpv
```

## Via Environment Variable

```bash
VK_INSTANCE_LAYERS=VK_LAYER_MCOF_device_filter \
    VK_DEVICE_FILTER=$regex \
    $command
```

# Installation

For advanced information for custom setups, see [the vulkan docs on layers](https://vulkan.lunarg.com/doc/view/1.0.13.0/windows/layers.html).

## Known packages

| Distribution | Link | Packager |
| ------------ | ---- | -------- |
| Arch Linux | [`vulkan-device-filter-git`](https://aur.archlinux.org/packages/vulkan-device-filter-git) | @mcoffin |

## Installation with scripts

To install as an explicit layer by copying the target files

```
cd vulkan-device-filter-layer
cargo build --release
./install.sh
```

To install as an explicit layer, you can use symlinks via the provided script.

```
cd vulkan-device-filter-layer
cargo build --release
./install-link.sh release
```

## Manual installation

If you don't want to use the provided install script(s), you can move the target files manually.

```bash
target_dir=${target_dir:-~/.local/share/vulkan/explicit_layer.d}
cd vulkan-device-filter-layer
cargo build --release
install -D -m755 -t "$target_dir" ../target/release/libvulkan_device_filter_layer.so
install -D -m644 -t $target_dir VkLayer_device_filter.json
```
