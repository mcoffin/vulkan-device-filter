use vulkan_device_filter_layer::version::{
    VulkanSemanticVersion,
    SemanticVersion
};

#[test]
#[no_mangle]
fn vulkan_semantic_version_roundtrip() {
    let major = 1;
    let minor = 2;
    let patch = 122;
    let vk_version = VulkanSemanticVersion::new(major, minor, patch);
    assert_eq!(major, vk_version.major());
    assert_eq!(minor, vk_version.minor());
    assert_eq!(patch, vk_version.patch());
}
