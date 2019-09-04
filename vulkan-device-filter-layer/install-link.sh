#!/bin/bash

set -e
set -x

build_type=${1:-debug}
target_dir=~/.local/share/vulkan/implicit_layer.d

pushd $target_dir
for f in VkLayer_device_filter.json libVkLayer_device_filter.so; do
	if [ -a "$f" ]; then
		rm "$f"
	fi
done
popd

ln -s -f "$(pwd)/../target/$build_type/libvulkan_device_filter_layer.so" $target_dir/libVkLayer_device_filter.so
ln -s -f "$(pwd)/VkLayer_device_filter.json" $target_dir/VkLayer_device_filter.json
