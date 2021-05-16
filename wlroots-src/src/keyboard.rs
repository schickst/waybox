

use crate::wlr::*;
use crate::server::*;
use crate::utils::*;
use crate::*;



#[repr(C)]
pub struct Keyboard {
    pub server: *mut Server,
    pub device: *mut wlr_input_device,
    pub modifiers: wl_listener,
    pub key: wl_listener
}


impl Keyboard {

    pub unsafe fn handle_keybinding(&self, server: &mut Server, sym: xkb_keysym_t) -> bool {
        #[allow(non_upper_case_globals)]
        match sym {
            XKB_KEY_Escape => {
                wl_display_terminate(server.display);
            }
            XKB_KEY_F3 => {
                // Cycle to next view.
                if server.views.len() > 2 {
                    server.views_idx = (server.views_idx + 1) % server.views.len();
                    let current_view = &mut server.views[server.views_idx];
                    // TODO: This lifetime-break-with-pointer
                    // is TOTALLY FINE.  Nothing to see here, move along.
                    let current_surface = (*current_view.xdg_surface).surface;
                    focus_view(current_view, &mut *current_surface);
                }
            }
            _ => {
                return server.configuration.handle_keybinding(sym);
            }
        }
        true
    }
}



pub unsafe extern "C" fn keyboard_handle_modifiers(
    listener: *mut wl_listener,
    _data: *mut ffi::c_void,
) {
    let keyboard = &mut *wl_container_of!(listener, Keyboard, modifiers);
    wlr_seat_set_keyboard((*keyboard.server).seat, (*keyboard).device);
    wlr_seat_keyboard_notify_modifiers(
        (*keyboard.server).seat,
        &mut (*(*keyboard.device).__bindgen_anon_1.keyboard).modifiers,
    );
}


pub unsafe extern "C" fn keyboard_handle_key(listener: *mut wl_listener, data: *mut ffi::c_void) {
    let keyboard = &mut *(wl_container_of!(listener, Keyboard, key));
    let server = keyboard.server;
    let event = &*(data as *const wlr_event_keyboard_key);
    let seat = (*server).seat;
    let kb = (*keyboard.device).__bindgen_anon_1.keyboard;

    // Translate libinput keycode -> xkbcommon
    let keycode: u32 = event.keycode + 8;
    // This needs a pointer-to-pointer, yay
    let syms: *mut *const xkb_keysym_t = &mut ptr::null();
    let nsyms = xkb_state_key_get_syms((*kb).xkb_state, keycode, syms);
    let mut handled = false;
    let modifiers = wlr_keyboard_get_modifiers(kb);

    if (*server).configuration.matches_modifiers(modifiers)
        && event.state == wlr_key_state_WLR_KEY_PRESSED
    {
        for i in 0..nsyms {
            // TODO: Figure out what the heck is up with syms, actually
            handled = keyboard.handle_keybinding(&mut *server, **syms.offset(i as isize));
        }
    }

    if !handled {
        // Pass it along to the client
        wlr_seat_set_keyboard(seat, keyboard.device);
        wlr_seat_keyboard_notify_key(seat, event.time_msec, event.keycode, event.state);
    }
}
