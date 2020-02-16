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
use crate::vk;

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
        use libc_regex_sys::{
            RegcompFlags,
            RegcompFlagsBuilder,
        };
        let regex_flags: RegcompFlags = RegcompFlagsBuilder::default()
            .extended(true)
            .into();
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
                        Regex::new(&*name, regex_flags)
                            .err_side_effect(|e| warn!("Invalid regex in config: {:?}", name))
                            .ok()
                            .map(|pattern| pattern.is_match(p))
                    })
                    .unwrap_or(false)
            },
            MatchRule::AppInfo { name, engine, app_version, engine_version, api_version } => {
                use crate::{
                    dispatches,
                    VulkanHandle,
                };
                use libc_regex_sys::Regex;
                let application_infos = dispatches::application_infos().read().unwrap();
                if let Some(application_info) = application_infos.get(&instance.vulkan_handle_key()) {
                    debug!("application info: {:?}", application_info);
                    if let Some(name) = name.as_ref().and_then(maybe_pattern) {
                        if let Some(real_name) = application_info.application_name() {
                            if !name.is_match(real_name) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    if let Some(engine) = engine.as_ref().and_then(maybe_pattern) {
                        if let Some(real_engine) = application_info.engine_name() {
                            if !engine.is_match(real_engine) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    if let Some(app_version) = app_version.as_ref().and_then(maybe_pattern) {
                        if version_match_excludes(&app_version, application_info.application_version.as_opt()) {
                            return false;
                        }
                    }
                    if let Some(engine_version) = engine_version.as_ref().and_then(maybe_pattern) {
                        if version_match_excludes(&engine_version, application_info.engine_version.as_opt()) {
                            return false;
                        }
                    }
                    if let Some(api_version) = api_version.as_ref().and_then(maybe_pattern) {
                        if version_match_excludes(&api_version, application_info.application_version.as_opt()) {
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
    use libc_regex_sys::{
        Regex,
        RegcompFlagsBuilder,
    };
    let s = s.as_ref();
    Regex::new(s, RegcompFlagsBuilder::default().extended(true).into())
        .err_side_effect(|e| warn!("Invalid regex in config: {:?}", e))
        .ok()
}

fn version_match_excludes(pattern: &libc_regex_sys::Regex, version: Option<vk::VulkanSemanticVersion>) -> bool {
    version
        .map(|v| format!("{}", v))
        .map(|real_version| {
            if !pattern.is_match(&*real_version) {
                true
            } else {
                false
            }
        })
        .unwrap_or(true)
}
