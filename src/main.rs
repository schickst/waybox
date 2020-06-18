use std::ffi;
use std::mem;
use std::pin::Pin;
use std::ptr;
use std::time::{Duration, Instant};

mod wlr;

use wlr::*;

use lazy_static::lazy_static;
lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum CursorMode {
    Passthrough,
    Move,
    Resize
}

#[repr(C)]
pub struct Server {
    display: *mut wl_display,
    backend: *mut wlr_backend,
    renderer: *mut wlr_renderer,

    xdg_shell: *mut wlr_xdg_shell,
    new_xdg_surface: wl_listener,

    views: Vec<Pin<Box<View>>>,
    views_idx: usize,

    cursor: *mut wlr_cursor,
    cursor_mgr: *mut wlr_xcursor_manager,
    cursor_motion: wl_listener,
    cursor_motion_absolute: wl_listener,
    cursor_button: wl_listener,
    cursor_axis: wl_listener,
    cursor_frame: wl_listener,

    seat: *mut wlr_seat,
    new_input: wl_listener,
    request_cursor: wl_listener,
    keyboards: Vec<Pin<Box<Keyboard>>>,
    cursor_mode: CursorMode,
    grabbed_view: *mut View,
    grab_x: f64,
    grab_y: f64,
    grab_width: i32,
    grab_height: i32,
    resize_edges: u32,

    output_layout: *mut wlr_output_layout,
    outputs: Vec<Pin<Box<Output>>>,
    new_output: wl_listener
}

#[repr(C)]
pub struct Keyboard {

}

#[repr(C)]
pub struct Output {

}

#[repr(C)]
pub struct View {

}

struct RenderData {

}

macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    }
}

macro_rules! wl_container_of {
    ($ptr: expr, $container: ty, $field: ident) => {
        ($ptr as *mut u8).offset(-(offset_of!($container, $field) as isize)) as *mut $container
    }
}


fn main() {
    println!("Hello, waybox!");

    lazy_static::initialize(&START_TIME);

    unsafe {
        wlr_log_init(wlr_log_importance_WLR_DEBUG, None);

        // Initialize Server, nove to impl Server::new
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

        let mut server = Server {
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
            new_output
        };

        // FIXME

        // Create socket
        let socket = wl_display_add_socket_auto(server.display);

        if socket.is_null() {
            wlr_backend_destroy(server.backend);
            wl_display_destroy(server.display);
        }
        let socket_str: &str = ffi::CStr::from_ptr(socket).to_str().expect("Unlikely");

        // Start backend
        if !wlr_backend_start(server.backend) {
            wlr_backend_destroy(server.backend);
            wl_display_destroy(server.display);
        }

        // Set env var and log socket
        std::env::set_var("WAYLAND_DISPLAY", socket_str);

        let log_str = ffi::CString::new("Running Waybar Wayland Compositor on display %s").unwrap();
        _wlr_log(wlr_log_importance_WLR_INFO, log_str.as_ptr(), socket);

        wl_display_run(server.display);

        // Cleanup
        wl_display_destroy_clients(server.display);
        wl_display_destroy(server.display);


    }
}
