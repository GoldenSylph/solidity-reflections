#![allow(unused_macros)]
//! Utils for the commands crate

use cliclack::select;
use reflections_core::{Result, config::detect_config_location};

/// Interactively get the config location from the user.
///
/// On error, it will default to `Foundry`.
pub fn get_config_location() -> Result<crate::ConfigLocation> {
    let current = detect_config_location(".");

    Ok(crate::ConfigLocation::from(
        select("Select the config location to use:")
            .initial_value(current.as_ref().map_or("foundry", |loc| match loc {
                reflections_core::config::ConfigLocation::Foundry => "foundry",
                reflections_core::config::ConfigLocation::Reflections => "reflections",
            }))
            .item("foundry", "Using foundry.toml", "recommended")
            .item("reflections", "Using reflections.toml", "for non-foundry projects")
            .interact()?
            .parse::<reflections_core::config::ConfigLocation>()
            .expect("all options should be valid variants of the ConfigLocation enum"),
    ))
}

macro_rules! intro {
    ($msg:expr) => {
        if $crate::TUI_ENABLED.load(::std::sync::atomic::Ordering::Relaxed) {
            ::cliclack::intro($msg).ok();
        }
    };
}

macro_rules! outro {
    ($msg:expr) => {
        if $crate::TUI_ENABLED.load(::std::sync::atomic::Ordering::Relaxed) {
            ::cliclack::outro($msg).ok();
        }
    };
}

macro_rules! outro_cancel {
    ($msg:expr) => {
        if $crate::TUI_ENABLED.load(::std::sync::atomic::Ordering::Relaxed) {
            ::cliclack::outro_cancel($msg).ok();
        }
    };
}

macro_rules! step {
    ($msg:expr) => {
        if $crate::TUI_ENABLED.load(::std::sync::atomic::Ordering::Relaxed) {
            ::cliclack::log::step($msg).ok();
        }
    };
}

macro_rules! remark {
    ($msg:expr) => {
        if $crate::TUI_ENABLED.load(::std::sync::atomic::Ordering::Relaxed) {
            ::cliclack::log::remark($msg).ok();
        }
    };
    ($fmt:expr, $($arg:expr),+) => {
        if $crate::TUI_ENABLED.load(::std::sync::atomic::Ordering::Relaxed) {
            ::cliclack::log::remark(format!($fmt, $($arg),+)).ok();
        }
    };
}

macro_rules! success {
    ($msg:expr) => {
        if $crate::TUI_ENABLED.load(::std::sync::atomic::Ordering::Relaxed) {
            ::cliclack::log::success($msg).ok();
        }
    };
    ($fmt:expr, $($arg:expr),+) => {
        if $crate::TUI_ENABLED.load(::std::sync::atomic::Ordering::Relaxed) {
            ::cliclack::log::success(format!($fmt, $($arg),+)).ok();
        }
    };
}

pub(crate) use intro;
pub(crate) use outro;
pub(crate) use outro_cancel;
pub(crate) use remark;
pub(crate) use step;
pub(crate) use success;
