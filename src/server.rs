
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

                configuration: Configuration::from_file("config.json")
            }
        }
    }

    pub unsafe fn new_pointer(&self, device: *mut wlr_input_device) {
        wlr_cursor_attach_input_device(self.cursor, device);
    }

    pub unsafe fn new_keyboard(&mut self, device: *mut wlr_input_device) {
        let mut modifiers: wl_listener = mem::zeroed();
        modifiers.notify = Some(keyboard_handle_modifiers);

        let mut key: wl_listener = mem::zeroed();
        key.notify = Some(keyboard_handle_key);

        let mut keyboard = Box::pin(Keyboard {
            server: self,
            device,
            modifiers,
            key,
        });

        // Prepare a default keymap and assign it to the keyboard
        let mut rules: xkb_rule_names = mem::zeroed();
        let context = xkb_context_new(xkb_context_flags_XKB_CONTEXT_NO_FLAGS);
        // Apparently xkb_map_new_from_names got renamed at some point?
        let keymap = xkb_keymap_new_from_names(
            context,
            &mut rules,
            xkb_keymap_compile_flags_XKB_KEYMAP_COMPILE_NO_FLAGS,
        );

        wlr_keyboard_set_keymap((*device).__bindgen_anon_1.keyboard, keymap);
        xkb_keymap_unref(keymap);
        xkb_context_unref(context);
        wlr_keyboard_set_repeat_info((*device).__bindgen_anon_1.keyboard, 25, 600);

        wl_signal_add(
            &mut (*(*device).__bindgen_anon_1.keyboard).events.modifiers,
            &mut (*keyboard).modifiers,
        );
        wl_signal_add(
            &mut (*(*device).__bindgen_anon_1.keyboard).events.key,
            &mut (*keyboard).key,
        );
        wlr_seat_set_keyboard(self.seat, device);
        self.keyboards.push(keyboard);
    }

}