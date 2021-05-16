#[macro_use] extern crate serde_derive;

use std::env;
use std::ffi;
use std::mem;
use std::pin::Pin;
use std::process::Command;
use std::ptr;
use std::time::Instant;

mod wlr;
mod config;
mod keyboard;
mod output;
mod server;
mod utils;
mod view;

use wlr::*;
use config::*;
use keyboard::*;
use output::*;
use server::*;
use utils::*;
use view::*;


use lazy_static::lazy_static;
lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

// ----------------





unsafe extern "C" fn server_new_input(listener: *mut wl_listener, data: *mut ffi::c_void) {
    let server = &mut *(wl_container_of!(listener, Server, new_input));
    let device = &mut *(data as *mut wlr_input_device);
    #[allow(non_upper_case_globals)]
    match device.type_ {
        wlr_input_device_type_WLR_INPUT_DEVICE_KEYBOARD => server.new_keyboard(device),
        wlr_input_device_type_WLR_INPUT_DEVICE_POINTER => server.new_pointer(device),
        _ => (),
    }
    // Let the wlr_seat know what our capabilities are,
    // we assume we always have a pointer.
    let caps: u32 = if server.keyboards.is_empty() {
        wl_seat_capability_WL_SEAT_CAPABILITY_POINTER
    } else {
        wl_seat_capability_WL_SEAT_CAPABILITY_POINTER
            | wl_seat_capability_WL_SEAT_CAPABILITY_KEYBOARD
    };
    wlr_seat_set_capabilities(server.seat, caps);
}

unsafe extern "C" fn seat_request_cursor(listener: *mut wl_listener, data: *mut ffi::c_void) {
    let server = &mut *(wl_container_of!(listener, Server, request_cursor));
    let event = &mut *(data as *mut wlr_seat_pointer_request_set_cursor_event);
    let focused_client = (*server.seat).pointer_state.focused_client;
    // Check that the client we got it from is the one that actually has focus
    if focused_client == event.seat_client {
        wlr_cursor_set_surface(
            server.cursor,
            event.surface,
            event.hotspot_x,
            event.hotspot_y,
        );
    }
}


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

unsafe extern "C" fn server_cursor_motion_absolute(listener: *mut wl_listener, data: *mut ffi::c_void) {
    let server = &mut *wl_container_of!(listener, Server, cursor_motion_absolute);
    let event = &mut *(data as *mut wlr_event_pointer_motion_absolute);

    wlr_cursor_warp_absolute(server.cursor, event.device, event.x, event.y);
    process_cursor_motion(server, event.time_msec);
}

unsafe extern "C" fn server_cursor_button(listener: *mut wl_listener, data: *mut ffi::c_void) {
    let server = &mut *wl_container_of!(listener, Server, cursor_button);
    let event = &mut *(data as *mut wlr_event_pointer_button);
    wlr_seat_pointer_notify_button(server.seat, event.time_msec, event.button, event.state);
    if let Some((view, surface, _sx, _sy)) =
        desktop_view_at(server, (*server.cursor).x, (*server.cursor).y)
    {
        if event.state == wlr_button_state_WLR_BUTTON_RELEASED {
            server.cursor_mode = CursorMode::Passthrough;
        } else {
            focus_view(&mut *view, &mut *surface);
        }
    }
}

unsafe extern "C" fn server_cursor_axis(listener: *mut wl_listener, data: *mut ffi::c_void) {
    let server = &mut *wl_container_of!(listener, Server, cursor_axis);
    let event = &mut *(data as *mut wlr_event_pointer_axis);
    wlr_seat_pointer_notify_axis(
        server.seat,
        event.time_msec,
        event.orientation,
        event.delta,
        event.delta_discrete,
        event.source,
    )
}

unsafe extern "C" fn server_cursor_frame(listener: *mut wl_listener, _data: *mut ffi::c_void) {
    let server = &mut *wl_container_of!(listener, Server, cursor_frame);
    wlr_seat_pointer_notify_frame(server.seat);
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
    let view = wl_container_of!(listener, View, map);
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


unsafe extern "C" fn handle_layer_shell_surface(listener: *mut wlr::wl_listener, data: *mut ffi::c_void) {
    println!("handle_layer_shell_surface called");
}



fn main() {

    let configuration = Configuration::from_file("config.json");

    env::set_var("XKB_DEFAULT_LAYOUT", &configuration.keyboard.layout);
    env::set_var("XKB_DEFAULT_VARIANT", &configuration.keyboard.variant);
    env::set_var("XKB_DEFAULT_MODEL", &configuration.keyboard.model);

    println!("Hello, waybox!");

    lazy_static::initialize(&START_TIME);

    unsafe {
        wlr_log_init(wlr_log_importance_WLR_DEBUG, None);

        // Initialize Server
        let mut server = Server::new(configuration);

        wlr_renderer_init_wl_display(server.renderer, server.display);
        wlr_compositor_create(server.display, server.renderer);
        wlr_data_device_manager_create(server.display);

        server.new_output.notify = Some(server_new_output);
        wl_signal_add(&mut (*server.backend).events.new_output, &mut server.new_output);

        wlr_xdg_output_manager_v1_create(&mut (*server.display), server.output_layout);


        server.layer_shell_surface.notify = Some(handle_layer_shell_surface);
        wl_signal_add(&mut (*server.layer_shell).events.new_surface,&mut server.layer_shell_surface);



        server.new_xdg_surface.notify = Some(server_new_xdg_surface);
        wl_signal_add(&mut (*server.xdg_shell).events.new_surface, &mut server.new_xdg_surface);


        // Cursor
        wlr_cursor_attach_output_layout(server.cursor, server.output_layout);
        wlr_xcursor_manager_load(server.cursor_mgr, 1.0);

        server.cursor_motion.notify = Some(server_cursor_motion);
        wl_signal_add(&mut (*server.cursor).events.motion, &mut server.cursor_motion);

        server.cursor_motion_absolute.notify = Some(server_cursor_motion_absolute);
        wl_signal_add(&mut (*server.cursor).events.motion_absolute, &mut server.cursor_motion_absolute);

        server.cursor_button.notify = Some(server_cursor_button);
        wl_signal_add(&mut (*server.cursor).events.button, &mut server.cursor_button);

        server.cursor_axis.notify = Some(server_cursor_axis);
        wl_signal_add(&mut (*server.cursor).events.axis, &mut server.cursor_axis);

        server.cursor_frame.notify = Some(server_cursor_frame);
        wl_signal_add(&mut (*server.cursor).events.frame, &mut server.cursor_frame);


        // Seat
        server.new_input.notify = Some(server_new_input);
        wl_signal_add(&mut (*server.backend).events.new_input, &mut server.new_input);

        let seat_name = ffi::CString::new("seat0").expect("Seat name error");
        server.seat = wlr_seat_create(server.display, seat_name.as_ptr());

        server.request_cursor.notify = Some(seat_request_cursor);
        wl_signal_add(&mut (*server.cursor).events.frame, &mut server.cursor_frame);


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

        let log_str = ffi::CString::new("Running waybox Wayland Compositor on display %s").unwrap();
        _wlr_log(wlr_log_importance_WLR_INFO, log_str.as_ptr(), socket);

        // missing something?
        Command::new("/bin/sh").arg("-c").arg("alacritty").spawn().expect("alacritty failed to launch");

        wl_display_run(server.display);

        // Cleanup
        wl_display_destroy_clients(server.display);
        wl_display_destroy(server.display);
    }
}
