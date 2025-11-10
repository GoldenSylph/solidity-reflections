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

macro_rules! define_cliclack_macro {
    ($name:ident, $path:path) => {
        macro_rules! $name {
            ($expression:expr) => {
                if $crate::TUI_ENABLED.load(::std::sync::atomic::Ordering::Relaxed) {
                    $path($expression).ok();
                }
            };
        }
    };
}

define_cliclack_macro!(intro, ::cliclack::intro);
define_cliclack_macro!(note, ::cliclack::note);
define_cliclack_macro!(outro, ::cliclack::outro);
define_cliclack_macro!(outro_cancel, ::cliclack::outro_cancel);
define_cliclack_macro!(outro_note, ::cliclack::outro_note);
define_cliclack_macro!(error, ::cliclack::log::error);
define_cliclack_macro!(info, ::cliclack::log::info);
define_cliclack_macro!(remark, ::cliclack::log::remark);
define_cliclack_macro!(step, ::cliclack::log::step);
define_cliclack_macro!(success, ::cliclack::log::success);
define_cliclack_macro!(warning, ::cliclack::log::warning);

#[allow(unused_imports)]
pub(crate) use error;
#[allow(unused_imports)]
pub(crate) use info;
pub(crate) use intro;
#[allow(unused_imports)]
pub(crate) use note;
pub(crate) use outro;
pub(crate) use outro_cancel;
#[allow(unused_imports)]
pub(crate) use outro_note;
pub(crate) use remark;
pub(crate) use step;
pub(crate) use success;
#[allow(unused_imports)]
pub(crate) use warning;
