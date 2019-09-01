use ::vulkan_sys as sys;
use std::ffi;
use std::mem;
use std::ptr;

pub struct DispatchTable {
    pfn_get_instance_proc_addr: sys::PFN_vkGetInstanceProcAddr,
    pfn_destroy_instance: sys::PFN_vkDestroyInstance,
    pfn_enumerate_device_extension_properties: sys::PFN_vkEnumerateDeviceExtensionProperties,
    pfn_enumerate_physical_devices: sys::PFN_vkEnumeratePhysicalDevices,
}

mod names {
    pub const GET_INSTANCE_PROC_ADDR: &'static [u8] = b"vkGetInstanceProcAddr\0";
    pub const DESTROY_INSTANCE: &'static [u8] = b"vkDestroyInstance\0";
    pub const ENUMERATE_DEVICE_EXTENSION_PROPERTIES: &'static [u8] = b"vkEnumerateDeviceExtensionProperties\0";
    pub const ENUMERATE_PHYSICAL_DEVICES: &'static [u8] = b"vkEnumeratePhysicalDevices\0";
}

impl DispatchTable {
    pub unsafe fn load<F>(mut load_fn: F) -> DispatchTable
        where F: FnMut(&ffi::CStr) -> sys::PFN_vkVoidFunction
    {
        let mut load = move |name: &[u8]| load_fn(ffi::CStr::from_bytes_with_nul_unchecked(name));
        DispatchTable {
            pfn_get_instance_proc_addr: mem::transmute(load(&names::GET_INSTANCE_PROC_ADDR)),
            pfn_destroy_instance: mem::transmute(load(&names::DESTROY_INSTANCE)),
            pfn_enumerate_device_extension_properties: mem::transmute(load(&names::ENUMERATE_DEVICE_EXTENSION_PROPERTIES)),
            pfn_enumerate_physical_devices: mem::transmute(load(&names::ENUMERATE_PHYSICAL_DEVICES)),
        }
    }

    pub unsafe fn destroy_instance(&self, instance: sys::VkInstance, allocation_callbacks: Option<&sys::VkAllocationCallbacks>) {
        let allocation_callbacks = allocation_callbacks
            .map(|cbs| cbs as *const sys::VkAllocationCallbacks)
            .unwrap_or(ptr::null());
        self.pfn_destroy_instance.unwrap()(instance, allocation_callbacks);
    }
}
