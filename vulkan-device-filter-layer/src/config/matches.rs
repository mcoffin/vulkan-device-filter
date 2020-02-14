use vulkan_sys as sys;

use super::MatchRule;

use log::{
    debug,
    warn,
};
use std::{
    env,
    io::{
        self,
        Write,
    },
};

trait ResultExt<T, E> {
    fn value_side_effect<F>(self, f: F) -> Self
    where
        F: FnOnce(&T);

    fn err_side_effect<F>(self, f: F) -> Self
    where
        F: FnOnce(&E);
}

impl<A, B> ResultExt<A, B> for Result<A, B> {
    #[inline]
    fn value_side_effect<F>(self, f: F) -> Self
    where
        F: FnOnce(&A),
    {
        self.map(move |v| {
            f(&v);
            v
        })
    }

    #[inline]
    fn err_side_effect<F>(self, f: F) -> Self
    where
        F: FnOnce(&B),
    {
        self.map_err(move |e| {
            f(&e);
            e
        })
    }
}

trait OptionExt<A> {
    fn and_then_zip<B, F>(self, f: F) -> Option<(A, B)>
    where
        F: FnOnce() -> Option<B>;
}

impl<A> OptionExt<A> for Option<A> {
    #[inline]
    fn and_then_zip<B, F>(self, f: F) -> Option<(A, B)>
    where
        F: FnOnce() -> Option<B>
    {
        self.and_then(move |v| f().map(|b| (v, b)))
    }
}

pub trait InstanceMatch {
    fn is_match(&self, instance: sys::VkInstance) -> bool;
}

impl InstanceMatch for MatchRule {
    fn is_match(&self, instance: sys::VkInstance) -> bool {
        match self {
            MatchRule::Executable { name } => {
                use libc_regex_sys::Regex;
                env::current_exe()
                    .value_side_effect(|v| debug!("executable: {:?}", v))
                    .err_side_effect(|e| warn!("Could not get current executable name: {:?}", e))
                    .ok()
                    .as_ref()
                    .and_then(|p| p.to_str())
                    .and_then(|p| {
                        Regex::new(&*name, libc_regex_sys::sys::REG_EXTENDED as i32)
                            .err_side_effect(|e| warn!("Invalid regex in config: {:?}", name))
                            .ok()
                            .map(|pattern| pattern.is_match(p))
                    })
                    .unwrap_or(false)
            },
            MatchRule::AppInfo { name, engine } => {
                use crate::{
                    dispatches,
                    VulkanHandle,
                };
                use libc_regex_sys::Regex;
                let application_infos = dispatches::application_infos().read().unwrap();
                if let Some(application_info) = application_infos.get(&instance.vulkan_handle_key()) {
                    debug!("application info: {:?}", application_info);
                    let name = name.as_ref().and_then(maybe_pattern);
                    if let Some((name, real_name)) = name.and_then_zip(|| application_info.engine_name()) {
                        if !name.is_match(real_name) {
                            return false;
                        }
                    }
                    let engine = engine.as_ref().and_then(maybe_pattern);
                    if let Some((engine, real_engine)) = engine.and_then_zip(|| application_info.engine_name()) {
                        if !engine.is_match(real_engine) {
                            return false;
                        }
                    }
                    return true;
                }
                false
            },
            MatchRule::And { rules } => {
                rules.iter()
                    .map(|rule| rule.is_match(instance))
                    .fold(true, |a, b| a && b)
            },
            MatchRule::Or { rules } => {
                rules.iter()
                    .map(|rule| rule.is_match(instance))
                    .fold(false, |a, b| a || b)
            },
        }
    }
}
fn maybe_pattern<S: AsRef<str>>(s: S) -> Option<libc_regex_sys::Regex> {
    use libc_regex_sys::Regex;
    let s = s.as_ref();
    Regex::new(s, libc_regex_sys::sys::REG_EXTENDED as i32)
        .err_side_effect(|e| warn!("Invalid regex in config: {:?}", e))
        .ok()
}
