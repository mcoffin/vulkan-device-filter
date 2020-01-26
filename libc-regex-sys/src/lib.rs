pub mod sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::{
    mem,
    ffi,
    ptr
};
use std::os::raw::c_int;

pub struct Regex(sys::regex_t);

impl Regex {
    pub fn new<S: AsRef<str>>(pattern: S, flags: c_int) -> Result<Regex, c_int> {
        let c_pattern = ffi::CString::new(pattern.as_ref()).unwrap();
        unsafe {
            let mut v: sys::regex_t = mem::zeroed();
            let status = sys::regcomp(&mut v as *mut sys::regex_t, c_pattern.as_ptr(), flags);
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
