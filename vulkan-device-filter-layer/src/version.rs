use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VulkanSemanticVersion(u32);

pub trait SemanticVersion {
    fn major(&self) -> u32;
    fn minor(&self) -> u32;
    fn patch(&self) -> u32;
}

impl VulkanSemanticVersion {
    pub fn new(mut major: u32, mut minor: u32, mut patch: u32) -> VulkanSemanticVersion {
        major = major & 0x3ff;
        minor = minor & 0x3ff;
        patch = patch & 0xfff;
        VulkanSemanticVersion::from_raw((major << 22) | (minor << 12) | patch)
    }

    #[inline(always)]
    pub fn from_raw(raw: u32) -> VulkanSemanticVersion {
        VulkanSemanticVersion(raw)
    }

    #[inline(always)]
    pub fn into_raw(self) -> u32 {
        self.into()
    }
}

impl Into<u32> for VulkanSemanticVersion {
    fn into(self) -> u32 {
        self.0
    }
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "bmi1"))]
const fn bextr_control32(start: u32, len: u32) -> u32 {
    (start & 0xff) | ((len & 0xff) << 8)
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "bmi1"))]
const MINOR_CONTROL: u32 = bextr_control32(12, 10);

impl SemanticVersion for VulkanSemanticVersion {
    #[inline(always)]
    fn major(&self) -> u32 {
        self.0 >> 22
    }

    #[inline(always)]
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "bmi1"))]
    fn minor(&self) -> u32 {
        #[cfg(target_arch = "x86")]
        use core::arch::x86::_bextr2_u32;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::_bextr2_u32;
        unsafe {
            _bextr2_u32(self.0, MINOR_CONTROL)
        }
    }

    #[inline(always)]
    #[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "bmi1")))]
    fn minor(&self) -> u32 {
        (self.0 >> 12) & 0x3ff
    }

    #[inline(always)]
    fn patch(&self) -> u32 {
        self.0 & 0xfff
    }
}

impl fmt::Display for VulkanSemanticVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major(), self.minor(), self.patch())
    }
}
impl fmt::Debug for VulkanSemanticVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VulkanSemanticVersion({}.{}.{})", self.major(), self.minor(), self.patch())
    }
}
