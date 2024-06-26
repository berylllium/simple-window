#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{LPARAM, WPARAM};

#[cfg(target_os = "windows")]
pub fn get_x_y_lparam(l_param: LPARAM) -> (i16, i16) {
    ((l_param & 0xFFFF) as i16, ((l_param >> 16) & 0xFFFF) as i16)
}

#[cfg(target_os = "windows")]
pub fn get_wheel_delta_wparam(w_param: WPARAM) -> i16 {
    ((w_param >> 16) & 0xFFFF) as i16
}
