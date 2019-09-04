#!/bin/bash

set -e
set -x

build_type=${1:-release}
target_dir=~/.local/share/vulkan/implicit_layer.d

if [ ! -d $target_dir ]; then
	mkdir -p $target_dir
fi

install -D -m755 ../target/release/libvulkan_device_filter_layer.so $target_dir/libVkLayer_device_filter.so
install -D -m644 -t $target_dir VkLayer_device_filter.json
