use vulkan_sys as sys;

pub use sys::VkInstanceCreateInfo as InstanceCreateInfo;
pub use sys::VkAllocationCallbacks as AllocationCallbacks;
pub use sys::VkInstance as Instance;
pub use sys::VkResult as Result;
pub use sys::VkLayerInstanceCreateInfo as LayerInstanceCreateInfo;
pub use sys::VkStructureType as StructureType;

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

    pub fn find_next<F>(&mut self, mut f: F) -> Option<&mut VkStructHead> where
        F: FnMut(&VkStructHead) -> bool
    {
        if f(self) {
            return Some(self);
        }

        while let Some(runner) = unsafe { self.p_next.as_mut() } {
            if f(runner) {
                return Some(runner);
            }
        }

        None
    }
}
