extern crate flag_builder;
use flag_builder::flag_builder;

pub mod sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::{
    mem,
    ffi,
    ptr
};
use std::os::raw::c_int;

/// Flags that can be passed to regexec
#[flag_builder(RegexecFlags, u32)]
#[repr(u32)]
pub enum RegexecFlagBits {
    NotBeginningOfLine = 0b1,
    NotEndOfLine = 0b10,
}

impl Default for RegexecFlags {
    fn default() -> RegexecFlags {
        RegexecFlags(0x0)
    }
}

impl Default for RegexecFlagsBuilder {
    fn default() -> RegexecFlagsBuilder {
        RegexecFlagsBuilder(0x0)
    }
}

#[flag_builder(RegcompFlags, u32)]
#[repr(u32)]
pub enum RegcompFlagBits {
    Extended = 0b1,
    IgnoreCase = 0b10,
    MatchNewline = 0b100,
    NoSub = 0b1000,
}

impl Default for RegcompFlags {
    fn default() -> RegcompFlags {
        RegcompFlags(0)
    }
}

impl Default for RegcompFlagsBuilder {
    fn default() -> RegcompFlagsBuilder {
        RegcompFlagsBuilder(0)
    }
}

pub struct Regex(sys::regex_t);

impl Regex {
    pub fn new<S: AsRef<str>>(pattern: S, flags: RegcompFlags) -> Result<Regex, c_int> {
        let flags: u32 = flags.into();
        let c_pattern = ffi::CString::new(pattern.as_ref()).unwrap();
        unsafe {
            let mut v: sys::regex_t = mem::zeroed();
            let status = sys::regcomp(&mut v as *mut sys::regex_t, c_pattern.as_ptr(), flags as c_int);
            if status != 0 {
                return Err(status);
            }
            Ok(Regex(v))
        }
    }

    pub fn is_match(&self, s: &str) -> bool {
        let c_s = ffi::CString::new(s).unwrap();
        let status = unsafe {
            sys::regexec(&self.0 as *const sys::regex_t, c_s.as_ptr(), 0, ptr::null_mut(), 0)
        };
        (status == 0)
    }
}

impl Drop for Regex {
    fn drop(&mut self) {
        unsafe {
            sys::regfree(&mut self.0 as *mut sys::regex_t);
        }
    }
}
