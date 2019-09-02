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

pub struct DeviceDispatchTable {
    pfn_get_device_proc_addr: sys::PFN_vkGetDeviceProcAddr,
    pfn_destroy_device: sys::PFN_vkDestroyDevice,
}

mod names {
    // Instance
    pub const DESTROY_INSTANCE: &'static [u8] = b"vkDestroyInstance\0";
    pub const ENUMERATE_DEVICE_EXTENSION_PROPERTIES: &'static [u8] = b"vkEnumerateDeviceExtensionProperties\0";
    pub const ENUMERATE_PHYSICAL_DEVICES: &'static [u8] = b"vkEnumeratePhysicalDevices\0";
}

impl DispatchTable {
    pub unsafe fn load<F>(pfn_get_instance_proc_addr: sys::PFN_vkGetInstanceProcAddr, mut load_fn: F) -> DispatchTable
        where F: FnMut(&ffi::CStr) -> sys::PFN_vkVoidFunction
    {
        let mut load = move |name: &[u8]| load_fn(ffi::CStr::from_bytes_with_nul_unchecked(name));
        DispatchTable {
            pfn_get_instance_proc_addr: pfn_get_instance_proc_addr,
            pfn_destroy_instance: mem::transmute(load(names::DESTROY_INSTANCE)),
            pfn_enumerate_device_extension_properties: mem::transmute(load(names::ENUMERATE_DEVICE_EXTENSION_PROPERTIES)),
            pfn_enumerate_physical_devices: mem::transmute(load(names::ENUMERATE_PHYSICAL_DEVICES)),
        }
    }

    pub unsafe fn enumerate_device_extension_properties(&self, physical_device: vulkan_sys::VkPhysicalDevice, layer_name: *const std::os::raw::c_char, property_count: *mut u32, properties: *mut vulkan_sys::VkExtensionProperties) -> vulkan_sys::VkResult {
        self.pfn_enumerate_device_extension_properties.unwrap()(physical_device, layer_name, property_count, properties)
    }

    pub unsafe fn get_instance_proc_addr(&self, instance: sys::VkInstance, name: *const std::os::raw::c_char) -> sys::PFN_vkVoidFunction {
        self.pfn_get_instance_proc_addr.unwrap()(instance, name)
    }

    pub unsafe fn destroy_instance(&self, instance: sys::VkInstance, allocation_callbacks: Option<&sys::VkAllocationCallbacks>) {
        let allocation_callbacks = allocation_callbacks
            .map(|cbs| cbs as *const sys::VkAllocationCallbacks)
            .unwrap_or(ptr::null());
        self.pfn_destroy_instance.unwrap()(instance, allocation_callbacks);
    }
}

impl DeviceDispatchTable {
    pub unsafe fn load<F>(pfn_get_device_proc_addr: sys::PFN_vkGetDeviceProcAddr, mut load_fn: F) -> DeviceDispatchTable where
        F: FnMut(&ffi::CStr) -> sys::PFN_vkVoidFunction
    {
        let mut load = move |name: &[u8]| load_fn(ffi::CStr::from_bytes_with_nul_unchecked(name));
        DeviceDispatchTable {
            pfn_get_device_proc_addr: pfn_get_device_proc_addr,
            pfn_destroy_device: mem::transmute(load(b"vkDestroyDevice")),
        }
    }

    pub unsafe fn get_device_proc_addr(&self, device: sys::VkDevice, name: *const std::os::raw::c_char) -> sys::PFN_vkVoidFunction {
        self.pfn_get_device_proc_addr.unwrap()(device, name)
    }

    pub unsafe fn destroy_device(&self, device: sys::VkDevice, allocation_callbacks: Option<&sys::VkAllocationCallbacks>) {
        let allocation_callbacks = allocation_callbacks
            .map(|cbs| cbs as *const sys::VkAllocationCallbacks)
            .unwrap_or(ptr::null());
        self.pfn_destroy_device.unwrap()(device, allocation_callbacks);
    }
}
