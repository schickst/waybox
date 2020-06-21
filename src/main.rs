use std::ffi;
use std::mem;
use std::pin::Pin;
use std::process::Command;
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

impl Server {
    fn new() -> Server {
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
                new_output
            }
        }
    }
}

#[repr(C)]
pub struct Keyboard {
    server: *mut Server,
    device: *mut wlr_input_device,
    modifiers: wl_listener,
    key: wl_listener
}

#[repr(C)]
pub struct Output {
    link: wl_list,
    server: *mut Server,
    wlr_output: *mut wlr_output,
    frame: wl_listener
}

#[repr(C)]
pub struct View {
    link: wl_list,
    server: *mut Server,
    xdg_surface: *mut wlr_xdg_surface,
    map: wl_listener,
    unmap: wl_listener,
    destroy: wl_listener,
    request_move: wl_listener,
    request_resize: wl_listener,
    mapped: bool,
    x: i32,
    y: i32
}

struct RenderData {
    output: *mut wlr_output,
    renderer: *mut wlr_renderer,
    view: *mut View,
    when: Duration
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

unsafe fn wl_signal_add(signal: *mut wl_signal, listener: *mut wl_listener) {
    wl_list_insert((*signal).listener_list.prev, &mut (*listener).link);
}

// ----------------

unsafe fn view_at(view: &mut View, lx: f64, ly: f64) -> Option<(*mut wlr_surface, f64, f64)> {
    let view_sx = lx - (view.x as f64);
    let view_sy = ly - (view.y as f64);

    let sx = &mut 0.0;
    let sy = &mut 0.0;

    let surface = wlr_xdg_surface_surface_at(view.xdg_surface, view_sx, view_sy, sx, sy);

    if surface != ptr::null_mut() {
        return Some((surface, *sx, *sy));
    }
    None
}

unsafe fn desktop_view_at(server: &mut Server, lx: f64, ly: f64) -> Option<(*mut View, *mut wlr_surface, f64, f64)> {
    for view in server.views.iter_mut() {
        match view_at(&mut *view, lx, ly) {
            None => (),
            Some((s, sx, sy)) => {
                return Some(((&mut **view) as *mut View, s, sx,sy));
            }
        }
    }
    None
}

unsafe fn process_cursor_move(server: &mut Server, _time: u32) {
    (*server.grabbed_view).x  = ((*server.cursor).x - server.grab_x) as i32;
    (*server.grabbed_view).y  = ((*server.cursor).y - server.grab_y) as i32;
}

unsafe fn process_cursor_resize(server: &mut Server, _time: u32) {
    let view = server.grabbed_view;
    let dx = (*server.cursor).x - server.grab_x;
    let dy = (*server.cursor).y - server.grab_y;
    let mut x: f64 = (*view).x as _;
    let mut y: f64 = (*view).y as _;
    let mut width: f64 = server.grab_width as _;
    let mut height: f64 = server.grab_height as _;

    if server.resize_edges & wlr_edges_WLR_EDGE_TOP != 0 {
        y = server.grab_y + dy;
        height -= dy;
        if height < 1.0 {
            y += height
        }
    } else if server.resize_edges & wlr_edges_WLR_EDGE_BOTTOM != 0 {
        height += dy;
    }

    if server.resize_edges & wlr_edges_WLR_EDGE_LEFT != 0 {
        x = server.grab_x + dx;
        width -= dx;
        if width < 1.0 {
            x += width;
        } else if server.resize_edges & wlr_edges_WLR_EDGE_RIGHT != 0 {
            width += dx
        }
    }
    (*view).x = x as i32;
    (*view).y = y as i32;
    wlr_xdg_toplevel_set_size((*view).xdg_surface, width as u32, height as u32);
}

unsafe fn process_cursor_passthrough(server: &mut Server, time: u32) {
    let seat = server.seat;

    if let Some((_view, surface, sx, sy)) = desktop_view_at(server, (*server.cursor).x, (*server.cursor).y) {
        if surface != ptr::null_mut() {
            let focus_changed = (*seat).pointer_state.focused_surface != surface;
            wlr_seat_pointer_notify_enter(seat, surface, sx, sy);

            if !focus_changed {
                wlr_seat_pointer_notify_motion(seat, time, sx, sy);
            }
        } else {
            wlr_seat_pointer_clear_focus(seat);
        }
    } else {
        let cursor_name = &ffi::CStr::from_bytes_with_nul_unchecked(b"left_ptr\0");
        wlr_xcursor_manager_set_cursor_image(server.cursor_mgr, cursor_name.as_ptr(), server.cursor);
    }
}

unsafe fn process_cursor_motion(server: &mut Server, time: u32) {
    if server.cursor_mode == CursorMode::Move {
        process_cursor_move(server, time);
        return;
    }
    if server.cursor_mode == CursorMode::Resize {
        process_cursor_resize(server, time);
        return;
    }
    process_cursor_passthrough(server, time);
}

unsafe extern "C" fn server_cursor_motion(listener: *mut wl_listener, data: *mut ffi::c_void) {
    let server = &mut *wl_container_of!(listener, Server, cursor_motion);
    let event = &mut *(data as *mut wlr_event_pointer_motion);

    wlr_cursor_move(server.cursor, event.device, event.delta_x, event.delta_y);
    process_cursor_motion(server, event.time_msec);
}



unsafe fn focus_view(view: &mut View, surface: &mut wlr_surface) {
    let server = view.server;
    let seat = (*server).seat;
    let prev_surface = (*seat).keyboard_state.focused_surface;

    if prev_surface == surface {
        return;
    }

    if prev_surface != ptr::null_mut() {
        let previous = wlr_xdg_surface_from_wlr_surface((*seat).keyboard_state.focused_surface);
        wlr_xdg_toplevel_set_activated(previous, false);
    }

    let keyboard = wlr_seat_get_keyboard(seat);

    if let Some((idx, _)) = (*view.server).views
                                                 .iter()
                                                 .enumerate()
                                                 .find(|(_idx, v)| &*(v.as_ref()) as *const View == view)
    {
        let v = (*server).views.remove(idx);
        (*server).views.insert(0, v);
    }

    wlr_xdg_toplevel_set_activated((*view).xdg_surface, true);
    wlr_seat_keyboard_notify_enter(seat, (*(*view).xdg_surface).surface, (*keyboard).keycodes.as_mut_ptr(), (*keyboard).num_keycodes, &mut (*keyboard).modifiers);
}

unsafe fn begin_interactive(view: &mut View, mode: CursorMode, edges: u32) {
    let server = &mut *view.server;
    let focused_surface = (*server.seat).pointer_state.focused_surface;

    if (*view.xdg_surface).surface != focused_surface {
        return;
    }

    server.grabbed_view = view;
    server.cursor_mode = mode;

    let mut geo_box = mem::zeroed();
    wlr_xdg_surface_get_geometry(view.xdg_surface, &mut geo_box);

    if mode == CursorMode::Move {
        server.grab_x = (*server.cursor).x - (view.x as f64);
        server.grab_x = (*server.cursor).y - (view.y as f64);
    } else {
        server.grab_x = (*server.cursor).x - (geo_box.x as f64);
        server.grab_x = (*server.cursor).y - (geo_box.y as f64);
    }

    server.grab_width = geo_box.width;
    server.grab_height = geo_box.height;
    server.resize_edges = edges;
}

unsafe extern "C" fn xdg_surface_map(listener: *mut wl_listener, _data: *mut ffi::c_void) {
    let view = wl_container_of!(listener, View, unmap);
    (*view).mapped = true;
    focus_view(&mut *view, &mut *(*(*view).xdg_surface).surface);
}

unsafe extern "C" fn xdg_surface_unmap(listener: *mut wl_listener, _data: *mut ffi::c_void) {
    let view = &mut *wl_container_of!(listener, View, unmap);
    view.mapped = false;
}

unsafe extern "C" fn xdg_surface_destroy(listener: *mut wl_listener, _data: *mut ffi::c_void) {
    let view = &mut *wl_container_of!(listener, View, destroy);

    // Ugh, I do not fully understand the type inference
    if let Some((idx, _)) = (*view.server).views
                                                 .iter()
                                                 .enumerate()
                                                 .find(|(_idx, v)| &*(v.as_ref()) as *const View == view)
    {
        (*view.server).views.remove(idx);
    } else {
        panic!("xdg_surface_destroy: Surface index not found");
    }
}


unsafe extern "C" fn xdg_toplevel_request_move(listener: *mut wl_listener, _data: *mut ffi::c_void) {
    let view = &mut *wl_container_of!(listener, View, request_move);
    begin_interactive(view, CursorMode::Move, 0);
}

unsafe extern "C" fn xdg_toplevel_request_resize(listener: *mut wl_listener, data: *mut ffi::c_void) {
    let view = &mut *wl_container_of!(listener, View, request_resize);
    let event = &mut *(data as *mut wlr_xdg_toplevel_resize_event);
    begin_interactive(view, CursorMode::Resize, event.edges);
}

unsafe extern "C" fn server_new_xdg_surface(listener: *mut wl_listener, data: *mut ffi::c_void) {
    let server = wl_container_of!(listener, Server, new_xdg_surface);
    let xdg_surface = data as *mut wlr_xdg_surface;

    if (*xdg_surface).role != wlr_xdg_surface_role_WLR_XDG_SURFACE_ROLE_TOPLEVEL {
        return;
    }

    let mut map: wl_listener = mem::zeroed();
    map.notify = Some(xdg_surface_map);

    let mut unmap: wl_listener = mem::zeroed();
    unmap.notify = Some(xdg_surface_unmap);

    let mut destroy: wl_listener = mem::zeroed();
    destroy.notify = Some(xdg_surface_destroy);

    let mut request_move: wl_listener = mem::zeroed();
    request_move.notify = Some(xdg_toplevel_request_move);

    let mut request_resize: wl_listener = mem::zeroed();
    request_resize.notify = Some(xdg_toplevel_request_resize);

    let link = mem::zeroed();

    let mut view = Box::pin(
        View {
            link,
            server,
            xdg_surface,

            map,
            unmap,
            destroy,
            request_move,
            request_resize,

            mapped: false,
            x: 0,
            y: 0
        }
    );

    wl_signal_add(&mut (*xdg_surface).events.map, &mut view.map);
    wl_signal_add(&mut (*xdg_surface).events.unmap, &mut view.unmap);
    wl_signal_add(&mut (*xdg_surface).events.destroy, &mut view.destroy);

    let toplevel = (*xdg_surface).__bindgen_anon_1.toplevel; // very pretty
    wl_signal_add(&mut (*toplevel).events.request_move, &mut view.request_move);
    wl_signal_add(&mut (*toplevel).events.request_resize, &mut view.request_resize);

    (*server).views.push(view);
}

unsafe extern "C" fn server_new_output(listener: *mut wl_listener, data: *mut ffi::c_void) {
    let server = &mut *(wl_container_of!(listener, Server, new_output));
    let wlr_output = &mut *(data as *mut wlr_output);

    if !(wl_list_empty(&wlr_output.modes) != 0) {
        let mode = wlr_output_preferred_mode(wlr_output);
        wlr_output_set_mode(wlr_output, mode);
        wlr_output_enable(wlr_output, true);

        if !wlr_output_commit(wlr_output) {
            return;
        }
    }

    // FIXME impl Output new()
    let mut output: Pin<Box<Output>> = Box::pin(mem::zeroed());
    output.wlr_output = wlr_output;
    output.server = server;
    output.frame.notify = Some(output_frame);

    wl_signal_add(&mut wlr_output.events.frame, &mut output.frame);
    server.outputs.push(output);

    wlr_output_layout_add_auto(server.output_layout, wlr_output);
}

unsafe extern "C" fn output_frame(listener: *mut wl_listener, _data: *mut ffi::c_void) {
    let output = &mut *(wl_container_of!(listener, Output, frame));
    let renderer = (*output.server).renderer;

    if !wlr_output_attach_render(output.wlr_output, ptr::null_mut()) {
        return;
    }

    let width = &mut 0;
    let height = &mut 0;
    wlr_output_effective_resolution(output.wlr_output, width, height);

    wlr_renderer_begin(renderer, *width, *height);
    let color = [0.1, 0.2, 0.3, 1.0];
    wlr_renderer_clear(renderer, color.as_ptr());

    // render back to front
    for view in (*output.server).views.iter_mut().rev() {
        if !view.mapped {
            continue;
        }

        let mut rdata = RenderData {
            output: output.wlr_output,
            view: &mut **view,
            renderer: renderer,
            when: START_TIME.elapsed()
        };

        wlr_xdg_surface_for_each_surface((*view).xdg_surface, Some(render_surface), (&mut rdata) as *mut _ as *mut ffi::c_void);
    }

    wlr_output_render_software_cursors(output.wlr_output, ptr::null_mut());
    wlr_renderer_end(renderer);
    wlr_output_commit(output.wlr_output);
}

unsafe extern "C" fn render_surface (surface: *mut wlr_surface, sx: i32, sy: i32, data: *mut ffi::c_void) {
    let rdata = &mut *(data as *mut RenderData);
    let view = rdata.view;
    let output = rdata.output;

    let texture = wlr_surface_get_texture(surface);
    if texture.is_null() {
        return;
    }

    let ox = &mut 0.0;
    let oy = &mut 0.0;
    wlr_output_layout_output_coords((*(*view).server).output_layout, output, ox, oy);
    *ox += ((*view).x + sx) as f64;
    *oy += ((*view).y + sy) as f64;

    let bx = wlr_box {
        x: (*ox * (*output).scale as f64) as i32,
        y: (*oy * (*output).scale as f64) as i32,
        width: ((*surface).current.width as f32 * (*output).scale) as i32,
        height: ((*surface).current.height as f32 * (*output).scale) as i32,
    };

    let matrix = [0.0; 9].as_mut_ptr();
    let transform = wlr_output_transform_invert((*surface).current.transform);

    wlr_matrix_project_box(matrix, &bx, transform, 0.0, (*output).transform_matrix.as_mut_ptr());
    wlr_render_texture_with_matrix(rdata.renderer, texture, matrix, 1.0);
    let ts = &mut timespec {
        tv_sec: rdata.when.as_secs() as i64,
        tv_nsec: rdata.when.subsec_nanos() as i64
    };
    wlr_surface_send_frame_done(surface, ts);
}


fn main() {
    println!("Hello, waybox!");

    lazy_static::initialize(&START_TIME);

    unsafe {
        wlr_log_init(wlr_log_importance_WLR_DEBUG, None);

        // Initialize Server
        let mut server = Server::new();

        wlr_renderer_init_wl_display(server.renderer, server.display);
        wlr_compositor_create(server.display, server.renderer);
        wlr_data_device_manager_create(server.display);

        server.new_output.notify = Some(server_new_output);
        wl_signal_add(&mut (*server.backend).events.new_output, &mut server.new_output);

        server.new_xdg_surface.notify = Some(server_new_xdg_surface);
        wl_signal_add(&mut (*server.xdg_shell).events.new_surface, &mut server.new_xdg_surface);

        wlr_cursor_attach_output_layout(server.cursor, server.output_layout);
        wlr_xcursor_manager_load(server.cursor_mgr, 1.0);

        server.cursor_motion.notify = Some(server_cursor_motion);
        wl_signal_add(&mut (*server.cursor).events.motion, &mut server.cursor_motion);



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

        // missing something?
        //Command::new("/bin/sh").arg("-c").arg("termite").spawn().expect("termite failed to launch");

        wl_display_run(server.display);

        // Cleanup
        wl_display_destroy_clients(server.display);
        wl_display_destroy(server.display);


    }
}
