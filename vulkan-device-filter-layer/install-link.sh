#!/bin/bash

set -e
set -x

while getopts ":i:" opt; do
	case ${opt} in
		i)
			layer_dir="implicit_layer.d"
			;;
		\?)
			echo "Invalid argument: -$OPTARG" >&2
			exit 1
			;;
		:)
			echo "Invalid argument: -$OPTARG requires an argument" >&2
			exit 1
			;;
	esac
done
shift $((OPTIND -1))

build_type="$1"
layer_dir=${layer_dir:-explicit_layer.d}
target_dir=~/.local/share/vulkan/$layer_dir

get_build_type() {
	local build_types=('debug' 'release')
	local bt
	for bt in ${build_types[@]}; do
		if [ -f ../target/$bt/libvulkan_device_filter_layer.so ]; then
			echo $bt
			return 0
		fi
	done
	echo "Unable to find built binary for (${build_types[@]})" >&2
	return 1
}

set -e

if [ -z "$build_type" ]; then
	build_type="$(get_build_type)"
fi

pushd $target_dir
for f in VkLayer_device_filter.json libVkLayer_device_filter.so; do
	if [ -a "$f" ]; then
		rm "$f"
	fi
done
popd

ln -s -f "$(pwd)/../target/$build_type/libvulkan_device_filter_layer.so" "$target_dir/libvulkan_device_filter_layer.so"
ln -s -f "$(pwd)/VkLayer_device_filter.json" $target_dir/VkLayer_device_filter.json
