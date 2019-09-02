pub extern crate ash;
pub extern crate vulkan_sys;
extern crate libc;

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
static mut DEVICE_DISPATCHES: Option<RwLock<BTreeMap<usize, layer::DeviceDispatchTable>>> = None;

#[link_name = "DeviceFilterLayer_CreateInstance"]
pub unsafe extern "system" fn create_instance(
    create_info: *const vk::InstanceCreateInfo,
    allocation_callbacks: *const vk::AllocationCallbacks,
    instance: *mut vk::Instance
) -> vk::Result {
    use layer::DispatchTable;

    println!("Filter layer: CreateInstance");

    let create_info = create_info.as_ref().unwrap();
    let next: &mut vk::VkStructHead = mem::transmute(create_info.pNext);
    let layer_create_info = next
        .find_next(|s| {
            if s.ty() == vulkan_sys::VkStructureType_VK_STRUCTURE_TYPE_LOADER_INSTANCE_CREATE_INFO {
                return true;
            }
            let info: &vk::LayerInstanceCreateInfo = mem::transmute(s);
            info.function == vulkan_sys::VkLayerFunction__VK_LAYER_LINK_INFO
        });
    if layer_create_info.is_none() {
        return vulkan_sys::VkResult_VK_ERROR_INITIALIZATION_FAILED;
    }
    let layer_create_info: &mut vk::LayerInstanceCreateInfo = mem::transmute(layer_create_info.unwrap());

    let gipa = layer_create_info.u.pLayerInfo.as_ref().unwrap().pfnNextGetInstanceProcAddr;
    layer_create_info.u.pLayerInfo = layer_create_info.u.pLayerInfo.as_ref().unwrap().pNext;

    let create_f = gipa.unwrap()(*instance, ffi::CStr::from_bytes_with_nul_unchecked(CREATE_INSTANCE_NAME).as_ptr());
    let create_f: vulkan_sys::PFN_vkCreateInstance = mem::transmute(create_f);

    let ret = create_f.unwrap()(create_info, allocation_callbacks, instance);
    if ret != vulkan_sys::VkResult_VK_SUCCESS {
        return ret;
    }

    if INSTANCE_DISPATCHES.is_none() {
        INSTANCE_DISPATCHES = Some(RwLock::new(BTreeMap::new()));
    }

    let dispatch_table = DispatchTable::load(gipa, |name| gipa.unwrap()(*instance, name.as_ptr()));
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

#[link_name = "DeviceFilterLayer_DestroyDevice"]
pub unsafe extern "system" fn destroy_device(
    device: vulkan_sys::VkDevice,
    allocation_callbacks: *const vk::AllocationCallbacks
) {
    let mut dispatches = DEVICE_DISPATCHES.as_ref().unwrap().write().unwrap();
    if let Some(dispatch) = dispatches.get(&mem::transmute(device)) {
        dispatch.destroy_device(device, allocation_callbacks.as_ref());
    }
    dispatches.remove(&mem::transmute(device));
}

#[link_name = "DeviceFilterLayer_CreateDevice"]
pub unsafe extern "system" fn create_device(
    physical_device: vulkan_sys::VkPhysicalDevice,
    create_info: *const vulkan_sys::VkDeviceCreateInfo,
    allocation_callbacks: *const vk::AllocationCallbacks,
    device: *mut vulkan_sys::VkDevice
) -> vk::Result {
    use layer::DeviceDispatchTable;

    let create_info = create_info.as_ref().unwrap();
    let next: &mut vk::VkStructHead = mem::transmute(create_info.pNext);
    let layer_create_info = next
        .find_next(|s| {
            if s.ty() == vulkan_sys::VkStructureType_VK_STRUCTURE_TYPE_LOADER_DEVICE_CREATE_INFO {
                return true;
            }
            let info: &vulkan_sys::VkLayerDeviceCreateInfo = mem::transmute(s);
            info.function == vulkan_sys::VkLayerFunction__VK_LAYER_LINK_INFO
        });
    if layer_create_info.is_none() {
        return vulkan_sys::VkResult_VK_ERROR_INITIALIZATION_FAILED;
    }
    let layer_create_info: &mut vulkan_sys::VkLayerDeviceCreateInfo = mem::transmute(layer_create_info.unwrap());

    let gipa = layer_create_info.u.pLayerInfo.as_ref().unwrap().pfnNextGetInstanceProcAddr;
    let gdpa = layer_create_info.u.pLayerInfo.as_ref().unwrap().pfnNextGetDeviceProcAddr;
    layer_create_info.u.pLayerInfo = layer_create_info.u.pLayerInfo.as_ref().unwrap().pNext;

    let create_f = gipa.unwrap()(mem::transmute(0 as usize), ffi::CStr::from_bytes_with_nul_unchecked(b"vkCreateDevice\0").as_ptr());
    let create_f: vulkan_sys::PFN_vkCreateDevice = mem::transmute(create_f);

    let ret = create_f.unwrap()(physical_device, create_info, allocation_callbacks, device);
    if ret != vulkan_sys::VkResult_VK_SUCCESS {
        return ret;
    }

    if DEVICE_DISPATCHES.is_none() {
        DEVICE_DISPATCHES = Some(RwLock::new(BTreeMap::new()));
    }

    let dispatch_table = DeviceDispatchTable::load(gdpa, |name| gdpa.unwrap()(*device, name.as_ptr()));
    let mut dispatches = DEVICE_DISPATCHES.as_ref().unwrap().write().unwrap();
    dispatches.insert(mem::transmute(*device), dispatch_table);

    vulkan_sys::VkResult_VK_SUCCESS
}

#[link_name = "DeviceFilterLayer_EnumerateInstanceLayerProperties"]
pub unsafe extern "system" fn enumerate_instance_layer_properties(
    property_count: *mut u32,
    properties: *mut vulkan_sys::VkLayerProperties
) -> vk::Result {
    use ffi::CStr;

    if !property_count.is_null() {
        *property_count = 1;
    }

    if !properties.is_null() {
        let properties = properties.as_mut().unwrap();

        libc::strcpy(properties.layerName.as_ptr() as *mut libc::c_char, CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_MCOF_device_filter\0").as_ptr());
        libc::strcpy(properties.description.as_ptr() as *mut libc::c_char, CStr::from_bytes_with_nul_unchecked(b"Device filter layer\0").as_ptr());

        properties.implementationVersion = 1;
        properties.specVersion = version::VulkanSemanticVersion::new(1, 0, 0).into();
    }

    return vulkan_sys::VkResult_VK_SUCCESS;
}

#[link_name = "DeviceFilterLayer_EnumerateDeviceLayerProperties"]
pub unsafe extern "system" fn enumerate_device_layer_properties(
    _physical_device: vulkan_sys::VkPhysicalDevice,
    property_count: *mut u32,
    properties: *mut vulkan_sys::VkLayerProperties
) -> vk::Result {
    enumerate_instance_layer_properties(property_count, properties)
}

#[link_name = "DeviceFilterLayer_EnumerateDeviceExtensionProperties"]
pub unsafe extern "system" fn enumerate_device_extension_properties(
    physical_device: vulkan_sys::VkPhysicalDevice,
    layer_name_orig: *const std::os::raw::c_char,
    property_count: *mut u32,
    properties: *mut vulkan_sys::VkExtensionProperties
) -> vk::Result {
    let layer_name = if layer_name_orig.is_null() {
        None
    } else {
        Some(ffi::CStr::from_ptr(layer_name_orig))
    };
    let layer_name = layer_name.map(|s| s.to_str().expect("Invalid UTF8 layer name"));
    if layer_name.is_none() || layer_name.filter(|&n| n != "VK_LAYER_MCOF_device_filter").is_some() {
        let physical_device_handle: usize = mem::transmute(physical_device);
        if physical_device_handle == 0 {
            return vulkan_sys::VkResult_VK_SUCCESS;
        }
        let dispatches = INSTANCE_DISPATCHES.as_ref().unwrap().read().unwrap();
        let dispatch = dispatches.get(&mem::transmute(physical_device)).unwrap();
        return dispatch.enumerate_device_extension_properties(physical_device, layer_name_orig, property_count, properties);
    }
    if !property_count.is_null() {
        *property_count = 0;
    }
    vulkan_sys::VkResult_VK_SUCCESS
}

#[link_name = "DeviceFilterLayer_EnumerateInstanceExtensionProperties"]
pub unsafe extern "system" fn enumerate_instance_extension_properties(
    layer_name: *const std::os::raw::c_char,
    property_count: *mut u32,
    _properties: *mut vulkan_sys::VkExtensionProperties
) -> vk::Result {
    let layer_name = if layer_name.is_null() {
        None
    } else {
        Some(ffi::CStr::from_ptr(layer_name))
    };
    let layer_name = layer_name.map(|s| s.to_str().expect("Invalid UTF8 layer name"));
    if layer_name.is_none() || layer_name.filter(|&n| n != "VK_LAYER_MCOF_device_filter").is_some() {
        return vulkan_sys::VkResult_VK_ERROR_LAYER_NOT_PRESENT;
    }
    if !property_count.is_null() {
        *property_count = 0;
    }
    vulkan_sys::VkResult_VK_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn DeviceFilterLayer_GetDeviceProcAddr(device: vulkan_sys::VkDevice, name: *const std::os::raw::c_char) -> vulkan_sys::PFN_vkVoidFunction {
    let n = ffi::CStr::from_ptr(name).to_str().unwrap();
    match n {
        "vkGetDeviceProcAddr" => Some(mem::transmute(&DeviceFilterLayer_GetDeviceProcAddr)),
        "vkEnumerateDeviceLayerProperties" => Some(mem::transmute(&enumerate_device_layer_properties)),
        "vkEnumerateDeviceExtensionProperties" => Some(mem::transmute(&enumerate_device_extension_properties)),
        "vkCreateDevice" => Some(mem::transmute(&create_device)),
        "vkDestroyDevice" => Some(mem::transmute(&destroy_device)),
        _ => {
            let dispatches = DEVICE_DISPATCHES.as_ref().unwrap().read().unwrap();
            let dispatch = dispatches.get(&mem::transmute(device)).unwrap();
            dispatch.get_device_proc_addr(device, name)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn DeviceFilterLayer_GetInstanceProcAddr(instance: vk::Instance, name: *const std::os::raw::c_char) -> vulkan_sys::PFN_vkVoidFunction {
    println!("DeviceFilterLayer: GetInstanceProcAddr");
    let n = ffi::CStr::from_ptr(name).to_str().unwrap();
    match n {
        "vkGetInstanceProcAddr" => Some(mem::transmute(&DeviceFilterLayer_GetInstanceProcAddr)),
        "vkEnumerateInstanceLayerProperties" => Some(mem::transmute(&enumerate_instance_layer_properties)),
        "vkEnumerateInstanceExtensionProperties" => Some(mem::transmute(&enumerate_instance_extension_properties)),
        "vkCreateInstance" => Some(mem::transmute(&create_instance)),
        "vkDestroyInstance" => Some(mem::transmute(&destroy_instance)),
        _ => {
            let dispatches = INSTANCE_DISPATCHES.as_ref().unwrap().read().unwrap();
            let dispatch = dispatches.get(&mem::transmute(instance)).unwrap();
            dispatch.get_instance_proc_addr(instance, name)
        }
    }
}
