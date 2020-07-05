use std::time::Duration;

use crate::wlr::*;
use crate::server::*;

#[repr(C)]
pub struct View {
    pub link: wl_list,
    pub server: *mut Server,
    pub xdg_surface: *mut wlr_xdg_surface,
    pub map: wl_listener,
    pub unmap: wl_listener,
    pub destroy: wl_listener,
    pub request_move: wl_listener,
    pub request_resize: wl_listener,
    pub mapped: bool,
    pub x: i32,
    pub y: i32
}

pub struct RenderData {
    pub output: *mut wlr_output,
    pub renderer: *mut wlr_renderer,
    pub view: *mut View,
    pub when: Duration
}