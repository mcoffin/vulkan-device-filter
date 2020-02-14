use vulkan_sys as sys;

pub use sys::VkInstanceCreateInfo as InstanceCreateInfo;
pub use sys::VkAllocationCallbacks as AllocationCallbacks;
pub use sys::VkInstance as Instance;
pub use sys::VkResult as Result;
pub use sys::VkLayerInstanceCreateInfo as LayerInstanceCreateInfo;
pub use sys::VkStructureType as StructureType;

use std::marker;
use crate::version::VulkanSemanticVersion;

#[repr(C)]
pub struct VkStructHead {
    s_type: StructureType,
    p_next: *mut VkStructHead
}

impl VkStructHead {
    #[inline(always)]
    pub fn ty(&self) -> StructureType {
        self.s_type
    }

    pub fn iter(&self) -> StructIter {
        StructIter {
            runner: Some(self)
        }
    }

    pub fn iter_mut(&mut self) -> StructIterMut {
        StructIterMut {
            runner: Some(self),
            _marker: marker::PhantomData {}
        }
    }

    pub fn find_next<F>(&mut self, mut f: F) -> Option<&mut VkStructHead> where
        F: FnMut(&VkStructHead) -> bool
    {
        if f(self) {
            return Some(self);
        }

        self.iter_mut()
            .find(|v| f(v))
    }
}

pub struct StructIterMut<'a> {
    runner: Option<*mut VkStructHead>,
    _marker: marker::PhantomData<&'a mut VkStructHead>,
}

impl<'a> Iterator for StructIterMut<'a> {
    type Item = &'a mut VkStructHead;

    fn next(&mut self) -> Option<Self::Item> {
        self.runner = self.runner
            .and_then(|p| unsafe { p.as_mut() })
            .map(|r| r.p_next)
            .and_then(|p| if p.is_null() {
                None
            } else {
                Some(p)
            });
        unsafe {
            self.runner
                .and_then(|p| p.as_mut())
        }
    }
}

pub struct StructIter<'a> {
    runner: Option<&'a VkStructHead>,
}

impl<'a> Iterator for StructIter<'a> {
    type Item = &'a VkStructHead;

    fn next(&mut self) -> Option<Self::Item> {
        self.runner = self.runner.as_ref().and_then(|r| unsafe { r.p_next.as_ref() });
        self.runner
    }
}

#[derive(Debug, Clone)]
pub struct ApplicationInfo<Name: AsRef<str>, EngineName: AsRef<str>> {
    application_name: Option<Name>,
    engine_name: Option<EngineName>,
    pub application_version: VulkanSemanticVersion,
    pub engine_version: VulkanSemanticVersion,
    pub api_version: VulkanSemanticVersion,
}

impl ApplicationInfo<String, String> {
    pub unsafe fn from_sys(application_info: &sys::VkApplicationInfo) -> Self {
        use std::ffi::CStr;
        let name = if !application_info.pApplicationName.is_null() {
            let s = CStr::from_ptr(application_info.pApplicationName);
            Some(s.to_string_lossy().to_string())
        } else {
            None
        };
        let engine = if !application_info.pEngineName.is_null() {
            let s = CStr::from_ptr(application_info.pEngineName);
            Some(s.to_string_lossy().to_string())
        } else {
            None
        };
        ApplicationInfo {
            application_name: name,
            engine_name: engine,
            application_version: VulkanSemanticVersion::from_raw(application_info.applicationVersion),
            engine_version: VulkanSemanticVersion::from_raw(application_info.engineVersion),
            api_version: VulkanSemanticVersion::from_raw(application_info.apiVersion),
        }
    }

    #[inline]
    pub fn application_name(&self) -> Option<&str> {
        self.application_name.as_ref().map(|s| s.as_ref())
    }

    #[inline]
    pub fn engine_name(&self) -> Option<&str> {
        self.engine_name.as_ref().map(|s| s.as_ref())
    }
}
