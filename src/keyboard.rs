

use crate::wlr::*;
use crate::server::*;


#[repr(C)]
pub struct Keyboard {
    pub server: *mut Server,
    pub device: *mut wlr_input_device,
    pub modifiers: wl_listener,
    pub key: wl_listener
}