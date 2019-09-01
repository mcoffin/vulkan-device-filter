pub extern crate ash;
pub extern crate vulkan_sys;

pub mod vk;
pub mod version;
mod layer;

use std::{
    ffi,
    mem
};

use std::collections::BTreeMap;
use std::sync::RwLock;

const CREATE_INSTANCE_NAME: &'static [u8] = b"vkCreateInstance\0";
static mut INSTANCE_DISPATCHES: Option<RwLock<BTreeMap<usize, layer::DispatchTable>>> = None;

#[link_name = "DeviceFilterLayer_CreateInstance"]
pub unsafe extern "system" fn create_instance(
    create_info: *const vk::InstanceCreateInfo,
    allocation_callbacks: *const vk::AllocationCallbacks,
    instance: *mut vk::Instance
) -> vk::Result {
    use layer::DispatchTable;

    let create_info = create_info.as_ref().unwrap();
    let next: &mut vk::VkStructHead = mem::transmute(create_info.pNext);
    let layer_create_info = next
        .find_next(|s| {
            if s.ty() == vulkan_sys::VkStructureType_VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO {
                return true;
            }
            let info: &vk::LayerInstanceCreateInfo = mem::transmute(s);
            info.function == vulkan_sys::VkLayerFunction__VK_LAYER_LINK_INFO
        });
    if layer_create_info.is_none() {
        return vulkan_sys::VkResult_VK_ERROR_INITIALIZATION_FAILED;
    }
    let layer_create_info: &mut vk::LayerInstanceCreateInfo = mem::transmute(layer_create_info.unwrap());
    layer_create_info.u.pLayerInfo = layer_create_info.u.pLayerInfo.as_ref().unwrap().pNext;

    let create_f = layer_create_info.u.pLayerInfo.as_ref().unwrap().pfnNextGetInstanceProcAddr.unwrap()(*instance, ffi::CStr::from_bytes_with_nul_unchecked(&CREATE_INSTANCE_NAME).as_ptr());
    let create_f: vulkan_sys::PFN_vkCreateInstance = mem::transmute(create_f);
    let ret = create_f.unwrap()(create_info, allocation_callbacks, instance);
    if ret != vulkan_sys::VkResult_VK_SUCCESS {
        return ret;
    }

    if INSTANCE_DISPATCHES.is_none() {
        INSTANCE_DISPATCHES = Some(RwLock::new(BTreeMap::new()));
    }

    let dispatch_table = DispatchTable::load(|name| layer_create_info.u.pLayerInfo.as_ref().unwrap().pfnNextGetInstanceProcAddr.unwrap()(*instance, name.as_ptr()));
    let mut dispatches = INSTANCE_DISPATCHES.as_ref().unwrap().write().unwrap();
    dispatches.insert(mem::transmute(*instance), dispatch_table);

    vulkan_sys::VkResult_VK_SUCCESS
}

#[link_name = "DeviceFilterLayer_DestroyInstance"]
pub unsafe extern "system" fn destroy_instance(
    instance: vk::Instance,
    allocation_callbacks: *const vk::AllocationCallbacks
) {
    let mut dispatches = INSTANCE_DISPATCHES.as_ref().unwrap().write().unwrap();
    if let Some(dispatch) = dispatches.get(&mem::transmute(instance)) {
        dispatch.destroy_instance(instance, allocation_callbacks.as_ref());
    }
    dispatches.remove(&mem::transmute(instance));
}
