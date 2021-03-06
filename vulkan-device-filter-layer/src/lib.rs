pub extern crate vulkan_sys;
extern crate libc;
extern crate regex;
extern crate serde;
extern crate serde_yaml;
extern crate log;
extern crate env_logger;
extern crate log4rs;

pub mod vk;
pub mod version;
mod layer;
mod config;

use config::Config;

use std::{
    env,
    ffi,
    iter,
    mem,
    ptr,
    slice
};

pub(crate) mod dispatches {
    use std::collections::BTreeMap;
    use std::sync::{self, RwLock};
    use super::{
        layer,
        vk,
    };

    pub type ApplicationInfo = vk::ApplicationInfo<String, String>;

    static mut INSTANCE_DISPATCHES: Option<RwLock<BTreeMap<usize, layer::DispatchTable>>> = None;
    static mut DEVICE_DISPATCHES: Option<RwLock<BTreeMap<usize, layer::DeviceDispatchTable>>> = None;
    static mut APPLICATION_INFOS: Option<RwLock<BTreeMap<usize, ApplicationInfo>>> = None;

    static INIT_I_DISPATCHES: sync::Once = sync::Once::new();
    static INIT_D_DISPATCHES: sync::Once = sync::Once::new();
    static INIT_APPLICATION_INFOS: sync::Once = sync::Once::new();

    pub fn application_infos() -> &'static RwLock<BTreeMap<usize, ApplicationInfo>> {
        unsafe {
            INIT_APPLICATION_INFOS.call_once(|| {
                APPLICATION_INFOS = Some(RwLock::new(BTreeMap::new()));
            });
            APPLICATION_INFOS.as_ref().unwrap()
        }
    }

    pub fn instances() -> &'static RwLock<BTreeMap<usize, layer::DispatchTable>> {
        unsafe {
            INIT_I_DISPATCHES.call_once(|| {
                INSTANCE_DISPATCHES = Some(RwLock::new(BTreeMap::new()));
            });
            INSTANCE_DISPATCHES.as_ref().unwrap()
        }
    }

    pub fn devices() -> &'static RwLock<BTreeMap<usize, layer::DeviceDispatchTable>> {
        unsafe {
            INIT_D_DISPATCHES.call_once(|| {
                DEVICE_DISPATCHES = Some(RwLock::new(BTreeMap::new()));
            });
            DEVICE_DISPATCHES.as_ref().unwrap()
        }
    }
}

trait DispatchTableExt {
    fn physical_device_properties(&self, physical_device: vulkan_sys::VkPhysicalDevice) -> vulkan_sys::VkPhysicalDeviceProperties;
}

impl DispatchTableExt for layer::DispatchTable {
    fn physical_device_properties(&self, physical_device: vulkan_sys::VkPhysicalDevice) -> vulkan_sys::VkPhysicalDeviceProperties {
        unsafe {
            let mut properties: vulkan_sys::VkPhysicalDeviceProperties = mem::zeroed();
            self.get_physical_device_properties(physical_device, &mut properties as *mut _);
            properties
        }
    }
}

trait PhysicalDevicePropertiesExt {
    fn get_name(&self) -> &ffi::CStr;
}

impl PhysicalDevicePropertiesExt for vulkan_sys::VkPhysicalDeviceProperties {
    fn get_name(&self) -> &ffi::CStr {
        unsafe {
            ffi::CStr::from_ptr(self.deviceName.as_ptr())
        }
    }
}

trait PhysicalDeviceGroupPropertiesExt {
    fn physical_devices(&self) -> &[vulkan_sys::VkPhysicalDevice];
}

impl PhysicalDeviceGroupPropertiesExt for vulkan_sys::VkPhysicalDeviceGroupProperties {
    fn physical_devices(&self) -> &[vulkan_sys::VkPhysicalDevice] {
        let count = self.physicalDeviceCount as usize;
        &self.physicalDevices[0..count]
    }
}

trait VkResultExt {
    fn is_success_or_incomplete(self) -> bool;
}

impl VkResultExt for vulkan_sys::VkResult {
    #[inline(always)]
    fn is_success_or_incomplete(self) -> bool {
        self == vulkan_sys::VkResult_VK_SUCCESS || self == vulkan_sys::VkResult_VK_INCOMPLETE
    }
}

fn get_filter(instance: vk::Instance) -> Option<libc_regex_sys::Regex> {
    use config::matches::InstanceMatch;
    use libc_regex_sys::{
        Regex,
        RegcompFlags,
        RegcompFlagsBuilder
    };
    let regex_flags: RegcompFlags = RegcompFlagsBuilder::default()
        .extended(true)
        .into();
    let env_filter = env::var("VK_DEVICE_FILTER")
        .ok()
        .and_then(|ref s| Regex::new(s, regex_flags).ok());
    if env_filter.is_some() {
        return env_filter;
    }
    Config::global().filters()
        .find(|f| f.match_rule().is_match(instance))
        .and_then(|f| Regex::new(f.filter(), regex_flags).ok())
}

#[link_name = "DeviceGroupFilter_EnumeratePhysicalDeviceGroups"]
pub unsafe extern "C" fn enumerate_physical_device_groups(
    instance: vk::Instance,
    physical_device_group_count: *mut u32,
    physical_device_groups: *mut vulkan_sys::VkPhysicalDeviceGroupProperties
) -> vk::Result {
    use std::collections::LinkedList;

    let physical_device_group_count = physical_device_group_count.as_mut().unwrap();
    let dispatch = {
        let dispatches = dispatches::instances().read().unwrap();
        let dispatch = dispatches.get(&instance.vulkan_handle_key()).map(Clone::clone);
        mem::drop(dispatches);
        dispatch.unwrap()
    };
    let status = dispatch.enumerate_physical_device_groups(instance, physical_device_group_count, physical_device_groups);
    if !status.is_success_or_incomplete() {
        return status;
    }
    // If devices is null, then we have to filter anyways to get the right # of potentially
    // available devices, so we have to allocate our own array. We shouldn't return these devices
    // anyways, as per
    // https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/vkEnumeratePhysicalDevices.html
    let mut buffer: Vec<vulkan_sys::VkPhysicalDeviceGroupProperties>;
    let groups = if physical_device_groups.is_null() {
        buffer = iter::repeat(mem::zeroed())
            .take(*physical_device_group_count as usize)
            .collect();
        let status = dispatch.enumerate_physical_device_groups(instance, physical_device_group_count, buffer.as_mut_slice().as_mut_ptr());
        if !status.is_success_or_incomplete() {
            return status;
        }
        buffer.as_mut_slice()
    } else {
        slice::from_raw_parts_mut(physical_device_groups, *physical_device_group_count as usize)
    };
    if let Some(filter) = get_filter(instance) {
        let group_matches = |group: &vulkan_sys::VkPhysicalDeviceGroupProperties| {
            let filtered_count = group.physical_devices()
                .iter()
                .map(|&d| dispatch.physical_device_properties(d))
                .filter(|p| p.get_name().to_str().as_ref().map(|s| filter.is_match(s)).unwrap_or(false))
                .map(|p| {
                    p
                })
                .count();
            filtered_count == group.physical_devices().len()
        };
        let filtered_groups: LinkedList<vulkan_sys::VkPhysicalDeviceGroupProperties> = groups.iter()
            .filter(move |&g| group_matches(g))
            .map(|&g| g)
            .collect();
        *physical_device_group_count = filtered_groups.len() as u32;
        for (i, g) in filtered_groups.iter().enumerate() {
            groups[i] = *g;
        }
        status
    } else {
        status
    }
}

#[link_name = "DeviceFilterLayer_EnumeratePhysicalDevices"]
pub unsafe extern "C" fn enumerate_physical_devices(
    instance: vk::Instance,
    physical_device_count: *mut u32,
    physical_devices: *mut vulkan_sys::VkPhysicalDevice
) -> vk::Result {
    use std::collections::LinkedList;
    let dispatch = {
        let dispatches = dispatches::instances().read().unwrap();
        let dispatch = dispatches.get(&instance.vulkan_handle_key()).map(Clone::clone);
        mem::drop(dispatches);
        dispatch.unwrap()
    };
    let mut status = dispatch.enumerate_physical_devices(instance, physical_device_count, physical_devices);
    if !status.is_success_or_incomplete() {
        return status;
    }

    // If devices is null, then we have to filter anyways to get the right # of potentially
    // available devices, so we have to allocate our own array. We shouldn't return these devices
    // anyways, as per
    // https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/vkEnumeratePhysicalDevices.html
    let mut buffer: Vec<vulkan_sys::VkPhysicalDevice>;
    let devices = if physical_devices.is_null() {
        buffer = iter::repeat(mem::zeroed())
            .take(*physical_device_count as usize)
            .collect();
        status = dispatch.enumerate_physical_devices(instance, physical_device_count, buffer.as_mut_slice().as_mut_ptr());
        if !status.is_success_or_incomplete() {
            return status;
        }
        buffer.as_mut_slice()
    } else {
        slice::from_raw_parts_mut(physical_devices, *physical_device_count as usize)
    };

    if let Some(filter) = get_filter(instance) {
        let filtered_devices: LinkedList<vulkan_sys::VkPhysicalDevice> = devices.iter()
            .map(|&device| (device, dispatch.physical_device_properties(device)))
            .filter_map(|(device, ref properties)| if properties.get_name().to_str().as_ref().map(|s| filter.is_match(s)).unwrap_or(false) {
                Some(device)
            } else {
                None
            })
            .collect();
        let mut filtered_count = 0;
        for (i, device) in filtered_devices.into_iter().enumerate() {
            devices[i] = device;
            filtered_count = i + 1;
        }
        *physical_device_count = filtered_count as u32;
        status
    } else {
        status
    }
}

#[allow(dead_code)]
fn display_pfn(f: vulkan_sys::PFN_vkVoidFunction) -> String {
    if let Some(v) = f {
        format!("{:#x}", v as usize)
    } else {
        "NULL".to_string()
    }
}

static INIT_LOGGER: std::sync::Once = std::sync::Once::new();

#[cfg(not(feature = "no_log"))]
fn init_logger() {
    use std::io::{
        self,
        Write,
    };
    println!("initializing logging");
    if let Some(path) = config::open_config_first("log4rs.yml") {
        if let Err(e) = log4rs::init_file(path, Default::default()) {
            let stderr = io::stderr();
            let mut stderr = stderr.lock();
            let _ = writeln!(&mut stderr, "Could not init log4rs, defaulting to using env_logger: {:?}", e);
        } else {
            return;
        }
    }
    #[cfg(debug_assertions)]
    {
        let stderr = io::stderr();
        let mut stderr = stderr.lock();
        let _ = writeln!(&mut stderr, "Could not find log4rs config file, defaulting to using env_logger");
    }
    env_logger::init();
}

#[cfg(feature = "no_log")]
fn init_logger() {}

#[link_name = "DeviceFilterLayer_CreateInstance"]
pub unsafe extern "C" fn create_instance(
    create_info: *const vk::InstanceCreateInfo,
    allocation_callbacks: *const vk::AllocationCallbacks,
    instance: *mut vk::Instance
) -> vk::Result {
    use layer::DispatchTable;

    INIT_LOGGER.call_once(|| init_logger());

    // println!("DeviceFilterLayer: CreateInstance");

    let create_info = create_info.as_ref().unwrap();
    let next = {
        let next: *mut vk::VkStructHead = mem::transmute(create_info.pNext);
        next.as_mut().expect("No pNext on create_info")
    };
    //println!("DeviceFilterLayer: CreateInstance: create_info.pNext = {:#x}", (next as *mut vk::VkStructHead) as usize);
    let layer_create_info = next
        .find_next(|s| {
            if s.ty() != vulkan_sys::VkStructureType_VK_STRUCTURE_TYPE_LOADER_INSTANCE_CREATE_INFO {
                return false;
            }
            let info: &vk::LayerInstanceCreateInfo = mem::transmute(s);
            info.function == vulkan_sys::VkLayerFunction__VK_LAYER_LINK_INFO
        });
    //println!("DeviceFilterLayer: CreateInstance: has_layer_create_info: {}", layer_create_info.is_some());
    if layer_create_info.is_none() {
        println!("DeviceFilterLayer: CreateDevice: bad create_info");
        return vulkan_sys::VkResult_VK_ERROR_INITIALIZATION_FAILED;
    }
    let layer_create_info: &mut vk::LayerInstanceCreateInfo = mem::transmute(layer_create_info.unwrap());
    //println!("DeviceFilterLayer: CreateInstance: layer_create_info = {:#x} (type = {:?}, function = {:?})", (layer_create_info as *mut vk::LayerInstanceCreateInfo) as usize, layer_create_info.sType, layer_create_info.function);

    let gipa = layer_create_info.u.pLayerInfo.as_ref().unwrap().pfnNextGetInstanceProcAddr;
    //println!("DeviceFilterLayer: CreateInstance: gipa = {}", display_pfn(mem::transmute(gipa)));
    layer_create_info.u.pLayerInfo = layer_create_info.u.pLayerInfo.as_ref().unwrap().pNext;

    //println!("DeviceFilterLayer: CreateInstance: instance = {:#x}", instance as usize);

    let create_instance_name = ffi::CStr::from_bytes_with_nul_unchecked(b"vkCreateInstance\0");

    //println!("DeviceFilterLayer: CreateInstance: create_instance_name = {:?}", create_instance_name);

    let create_f = gipa.unwrap()(mem::transmute(ptr::null::<usize>()), create_instance_name.as_ptr());
    let create_f: vulkan_sys::PFN_vkCreateInstance = mem::transmute(create_f);
    //println!("DeviceFilterLayer: CreateInstance: create_f = {}", display_pfn(mem::transmute(create_f)));

    let ret = create_f.unwrap()(create_info, allocation_callbacks, instance);
    if ret != vulkan_sys::VkResult_VK_SUCCESS {
        return ret;
    }

    let dispatch_table = DispatchTable::load(gipa, |name| gipa.unwrap()(*instance, name.as_ptr()));
    {
        let mut dispatches = dispatches::instances().write().unwrap();
        dispatches.insert((*instance).vulkan_handle_key(), dispatch_table);
    }
    {
        let application_info = create_info.pApplicationInfo.as_ref()
            .map(|info| vk::ApplicationInfo::from_sys(info));
        if let Some(application_info) = application_info {
            let mut dispatches = dispatches::application_infos().write().unwrap();
            dispatches.insert((*instance).vulkan_handle_key(), application_info);
        }
    }

    //println!("DeviceFilterLayer: CreateInstance: done");

    vulkan_sys::VkResult_VK_SUCCESS
}

#[link_name = "DeviceFilterLayer_DestroyInstance"]
pub unsafe extern "C" fn destroy_instance(
    instance: vk::Instance,
    allocation_callbacks: *const vk::AllocationCallbacks
) {
    let mut dispatches = dispatches::instances().write().unwrap();
    if let Some(dispatch) = dispatches.get(&mem::transmute(instance)) {
        dispatch.destroy_instance(instance, allocation_callbacks.as_ref());
    }
    dispatches.remove(&mem::transmute(instance));
}

#[link_name = "DeviceFilterLayer_DestroyDevice"]
pub unsafe extern "C" fn destroy_device(
    device: vulkan_sys::VkDevice,
    allocation_callbacks: *const vk::AllocationCallbacks
) {
    let mut dispatches = dispatches::devices().write().unwrap();
    if let Some(dispatch) = dispatches.get(&mem::transmute(device)) {
        dispatch.destroy_device(device, allocation_callbacks.as_ref());
    }
    dispatches.remove(&mem::transmute(device));
}

#[link_name = "DeviceFilterLayer_CreateDevice"]
pub unsafe extern "C" fn create_device(
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
            if s.ty() != vulkan_sys::VkStructureType_VK_STRUCTURE_TYPE_LOADER_DEVICE_CREATE_INFO {
                return false;
            }
            let info: &vulkan_sys::VkLayerDeviceCreateInfo = mem::transmute(s);
            info.function == vulkan_sys::VkLayerFunction__VK_LAYER_LINK_INFO
        });
    if layer_create_info.is_none() {
        println!("DeviceFilterLayer: CreateDevice: bad create_info");
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
        println!("DeviceFilterLayer: CreateDevice: Downstream failure: {}", ret);
        return ret;
    }

    let dispatch_table = DeviceDispatchTable::load(gdpa, |name| gdpa.unwrap()(*device, name.as_ptr()));
    let mut dispatches = dispatches::devices().write().unwrap();
    dispatches.insert((*device).vulkan_handle_key(), dispatch_table);

    vulkan_sys::VkResult_VK_SUCCESS
}

#[link_name = "DeviceFilterLayer_EnumerateInstanceLayerProperties"]
pub unsafe extern "C" fn enumerate_instance_layer_properties(
    property_count: *mut u32,
    properties: *mut vulkan_sys::VkLayerProperties
) -> vk::Result {
    use ffi::CStr;

    if !property_count.is_null() {
        *property_count = 1;
    }

    if !properties.is_null() {
        let properties = properties.as_mut().unwrap();

        #[cfg(target_arch = "x86")]
        {
            libc::strcpy(properties.layerName.as_ptr() as *mut libc::c_char, CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_MCOF_device_filter_32\0").as_ptr());
        }
        #[cfg(not(target_arch = "x86"))]
        {
            libc::strcpy(properties.layerName.as_ptr() as *mut libc::c_char, CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_MCOF_device_filter\0").as_ptr());
        }
        libc::strcpy(properties.description.as_ptr() as *mut libc::c_char, CStr::from_bytes_with_nul_unchecked(b"Device filter layer\0").as_ptr());

        properties.implementationVersion = 1;
        properties.specVersion = version::VulkanSemanticVersion::new(1, 0, 0).into();
    }

    return vulkan_sys::VkResult_VK_SUCCESS;
}

#[link_name = "DeviceFilterLayer_EnumerateDeviceLayerProperties"]
pub unsafe extern "C" fn enumerate_device_layer_properties(
    _physical_device: vulkan_sys::VkPhysicalDevice,
    property_count: *mut u32,
    properties: *mut vulkan_sys::VkLayerProperties
) -> vk::Result {
    enumerate_instance_layer_properties(property_count, properties)
}

fn is_device_filter_layer(n: &str) -> bool {
    n != "VK_LAYER_MCOF_device_filter" && n != "VK_LAYER_MCOF_device_filter_32"
}

#[link_name = "DeviceFilterLayer_EnumerateDeviceExtensionProperties"]
pub unsafe extern "C" fn enumerate_device_extension_properties(
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
    if layer_name.is_none() || layer_name.filter(|&n| is_device_filter_layer(n)).is_some() {
        let physical_device_handle: usize = mem::transmute(physical_device);
        if physical_device_handle == 0 {
            return vulkan_sys::VkResult_VK_SUCCESS;
        }
        let dispatches = dispatches::instances().read().unwrap();
        let dispatch = dispatches.get(&physical_device.vulkan_handle_key()).unwrap();
        return dispatch.enumerate_device_extension_properties(physical_device, layer_name_orig, property_count, properties);
    }
    if !property_count.is_null() {
        *property_count = 0;
    }
    vulkan_sys::VkResult_VK_SUCCESS
}

#[link_name = "DeviceFilterLayer_EnumerateInstanceExtensionProperties"]
pub unsafe extern "C" fn enumerate_instance_extension_properties(
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
    if layer_name.is_none() || layer_name.filter(|&n| is_device_filter_layer(n)).is_some() {
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
    //println!("DeviceFilterLayer: GetDeviceProcAddr: {}", n);
    let ret: vulkan_sys::PFN_vkVoidFunction = match n {
        "vkGetDeviceProcAddr" => {
            let pfn_get_device_proc_addr: vulkan_sys::PFN_vkGetDeviceProcAddr = Some(DeviceFilterLayer_GetDeviceProcAddr);
            mem::transmute(pfn_get_device_proc_addr)
        },
        "vkEnumerateDeviceLayerProperties" => {
            let pfn_enumerate_device_layer_properties: vulkan_sys::PFN_vkEnumerateDeviceLayerProperties = Some(enumerate_device_layer_properties);
            mem::transmute(pfn_enumerate_device_layer_properties)
        },
        "vkEnumerateDeviceExtensionProperties" => {
            let pfn_edep: vulkan_sys::PFN_vkEnumerateDeviceExtensionProperties = Some(enumerate_device_extension_properties);
            mem::transmute(pfn_edep)
        },
        "vkCreateDevice" => {
            let pfn_create_device: vulkan_sys::PFN_vkCreateDevice = Some(create_device);
            mem::transmute(pfn_create_device)
        },
        "vkDestroyDevice" => {
            let pfn_destroy_device: vulkan_sys::PFN_vkDestroyDevice = Some(destroy_device);
            mem::transmute(pfn_destroy_device)
        },
        _ => {

            let dispatches = dispatches::devices().read().unwrap();
            let dispatch = dispatches.get(&device.vulkan_handle_key())
                .expect(&format!("{}: device not yet registered: {:?}, {}", n, device, device.vulkan_handle_key()));
            dispatch.get_device_proc_addr(device, name)
        }
    };
    // if let Some(v) = ret.as_ref() {
    //     println!("    -> {:#x}", *v as usize);
    // } else {
    //     println!("    -> NULL");
    // }
    ret
}

mod lookup {
    use std::collections::BTreeMap;
    use std::mem;
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref INSTANCE_LOOKUP: BTreeMap<&'static str, vulkan_sys::PFN_vkVoidFunction> = {
            unsafe {
                let mut i_map = BTreeMap::new();
                let f: vulkan_sys::PFN_vkGetInstanceProcAddr = Some(DeviceFilterLayer_GetInstanceProcAddr);
                i_map.insert("vkGetInstanceProcAddr", mem::transmute(f));
                let f: vulkan_sys::PFN_vkEnumerateInstanceLayerProperties = Some(enumerate_instance_layer_properties);
                i_map.insert("vkEnumerateInstanceLayerProperties", mem::transmute(f));
                let f: vulkan_sys::PFN_vkEnumerateInstanceExtensionProperties = Some(enumerate_instance_extension_properties);
                i_map.insert("vkEnumerateInstanceExtensionProperties", mem::transmute(f));
                let f: vulkan_sys::PFN_vkCreateInstance = Some(create_instance);
                i_map.insert("vkCreateInstance", mem::transmute(f));
                let f: vulkan_sys::PFN_vkDestroyInstance = Some(destroy_instance);
                i_map.insert("vkDestroyInstance", mem::transmute(f));
                let f: vulkan_sys::PFN_vkGetDeviceProcAddr = Some(DeviceFilterLayer_GetDeviceProcAddr);
                i_map.insert("vkGetDeviceProcAddr", mem::transmute(f));
                let f: vulkan_sys::PFN_vkEnumerateDeviceLayerProperties = Some(enumerate_device_layer_properties);
                i_map.insert("vkEnumerateDeviceLayerProperties", mem::transmute(f));
                let f: vulkan_sys::PFN_vkEnumerateDeviceExtensionProperties = Some(enumerate_device_extension_properties);
                i_map.insert("vkEnumerateDeviceExtensionProperties", mem::transmute(f));
                let f: vulkan_sys::PFN_vkEnumeratePhysicalDevices = Some(enumerate_physical_devices);
                i_map.insert("vkEnumeratePhysicalDevices", mem::transmute(f));
                let f: vulkan_sys::PFN_vkEnumeratePhysicalDeviceGroups = Some(enumerate_physical_device_groups);
                ["vkEnumeratePhysicalDeviceGroups", "vkEnumeratePhysicalDeviceGroupsKHR"].iter().for_each(|k| {
                    i_map.insert(k, mem::transmute(f));
                });
                let f: vulkan_sys::PFN_vkCreateDevice = Some(create_device);
                i_map.insert("vkCreateDevice", mem::transmute(f));
                let f: vulkan_sys::PFN_vkDestroyDevice = Some(destroy_device);
                i_map.insert("vkDestroyDevice", mem::transmute(f));
                i_map
            }
        };
    }

    pub fn instance() -> &'static BTreeMap<&'static str, vulkan_sys::PFN_vkVoidFunction> {
        &INSTANCE_LOOKUP
    }
}

#[no_mangle]
pub unsafe extern "C" fn DeviceFilterLayer_GetInstanceProcAddr(instance: vk::Instance, name: *const std::os::raw::c_char) -> vulkan_sys::PFN_vkVoidFunction {
    let n = ffi::CStr::from_ptr(name).to_str().unwrap();
    lookup::instance()
        .get(&n)
        .map(|&p| p)
        .unwrap_or_else(|| {
            let dispatches = dispatches::instances().read().unwrap();
            dispatches
                .get(&instance.vulkan_handle_key())
                .and_then(|d| d.get_instance_proc_addr(instance, name))
        })
}

pub trait VulkanHandle {
    fn vulkan_handle_key(self) -> usize;
}

impl VulkanHandle for vk::Instance {
    fn vulkan_handle_key(self) -> usize {
        unsafe {
            let ptr: *mut usize = mem::transmute(self);
            *ptr
        }
    }
}

impl VulkanHandle for vulkan_sys::VkDevice {
    fn vulkan_handle_key(self) -> usize {
        unsafe {
            let ptr: *mut usize = mem::transmute(self);
            *ptr
        }
    }
}

impl VulkanHandle for vulkan_sys::VkPhysicalDevice {
    fn vulkan_handle_key(self) -> usize {
        unsafe {
            let ptr: *mut usize = mem::transmute(self);
            *ptr
        }
    }
}
