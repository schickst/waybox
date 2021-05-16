
use crate::wlr::*;
use crate::server::*;



#[repr(C)]
pub struct Output {
    pub link: wl_list,
    pub server: *mut Server,
    pub wlr_output: *mut wlr_output,
    pub frame: wl_listener
}


