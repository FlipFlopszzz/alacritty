use serde::{Deserialize, Deserializer, Serialize};

use alacritty_config_derive::{ConfigDeserialize, SerdeReplace};

use crate::config::bindings::{self, MouseBinding};
use crate::config::ui_config;

#[derive(ConfigDeserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Mouse {
    pub hide_when_typing: bool,
    /// Fork: maximum interval between clicks for them to register as a
    /// double/triple click, in milliseconds. Defaults to the system double
    /// click time on Windows (`GetDoubleClickTime`), 400 ms elsewhere.
    pub double_click_interval: u16,
    #[serde(skip_serializing)]
    pub bindings: MouseBindings,
}

impl Default for Mouse {
    fn default() -> Self {
        Self {
            hide_when_typing: false,
            double_click_interval: Self::default_double_click_interval(),
            bindings: MouseBindings(bindings::default_mouse_bindings()),
        }
    }
}

impl Mouse {
    fn default_double_click_interval() -> u16 {
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime;

            let millis = GetDoubleClickTime();
            if millis != 0 {
                return millis.min(u16::MAX as u32) as u16;
            }
        }
        400
    }
}

#[derive(SerdeReplace, Clone, Debug, PartialEq, Eq)]
pub struct MouseBindings(pub Vec<MouseBinding>);

impl Default for MouseBindings {
    fn default() -> Self {
        Self(bindings::default_mouse_bindings())
    }
}

impl<'de> Deserialize<'de> for MouseBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(ui_config::deserialize_bindings(deserializer, Self::default().0)?))
    }
}
