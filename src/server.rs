
use crate::wlr::*;
use crate::output::*;
use crate::utils::*;
use crate::*;




#[repr(C)]
pub struct Server {
    pub display: *mut wl_display,
    pub backend: *mut wlr_backend,
    pub renderer: *mut wlr_renderer,

    pub xdg_shell: *mut wlr_xdg_shell,
    pub new_xdg_surface: wl_listener,

    pub views: Vec<Pin<Box<View>>>,
    pub views_idx: usize,

    pub cursor: *mut wlr_cursor,
    pub cursor_mgr: *mut wlr_xcursor_manager,
    pub cursor_motion: wl_listener,
    pub cursor_motion_absolute: wl_listener,
    pub cursor_button: wl_listener,
    pub cursor_axis: wl_listener,
    pub cursor_frame: wl_listener,

    pub seat: *mut wlr_seat,
    pub new_input: wl_listener,
    pub request_cursor: wl_listener,
    pub keyboards: Vec<Pin<Box<Keyboard>>>,
    pub cursor_mode: CursorMode,
    pub grabbed_view: *mut View,
    pub grab_x: f64,
    pub grab_y: f64,
    pub grab_width: i32,
    pub grab_height: i32,
    pub resize_edges: u32,

    pub output_layout: *mut wlr_output_layout,
    pub outputs: Vec<Pin<Box<Output>>>,
    pub new_output: wl_listener,

    pub configuration: Configuration
}

impl Server {
    pub fn new() -> Server {
        unsafe {
            let display = wl_display_create();
            let backend = wlr_backend_autocreate(display, None);
            let renderer = wlr_backend_get_renderer(backend);

            let views = Vec::new();
            let xdg_shell = wlr_xdg_shell_create(display);
            let new_xdg_surface = mem::zeroed();

            let cursor = wlr_cursor_create();
            let cursor_axis = mem::zeroed();
            let cursor_button = mem::zeroed();
            let cursor_frame = mem::zeroed();
            let cursor_motion = mem::zeroed();
            let cursor_motion_absolute = mem::zeroed();
            let cursor_mgr = wlr_xcursor_manager_create(ptr::null(), 24);

            let seat = ptr::null_mut();
            let request_cursor = mem::zeroed();
            let new_input = mem::zeroed();

            let output_layout = wlr_output_layout_create();
            let new_output = mem::zeroed();

            Server {
                display,
                backend,
                renderer,

                views,
                views_idx: 0,
                xdg_shell,
                new_xdg_surface,

                cursor,
                cursor_axis,
                cursor_button,
                cursor_frame,
                cursor_motion,
                cursor_motion_absolute,
                cursor_mgr,

                seat,
                keyboards: Vec::new(),
                new_input,
                request_cursor,
                cursor_mode: CursorMode::Passthrough,
                grab_x: 0.0,
                grab_y: 0.0,
                grab_width: 0,
                grab_height: 0,
                resize_edges: 0,
                grabbed_view: ptr::null_mut(),

                output_layout,
                outputs: Vec::new(),
                new_output,

                configuration: Configuration::new()
            }
        }
    }
}