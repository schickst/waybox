use crate::wlr::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum CursorMode {
    Passthrough,
    Move,
    Resize
}


#[macro_export] macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    }
}

#[macro_export] macro_rules! wl_container_of {
    ($ptr: expr, $container: ty, $field: ident) => {
        ($ptr as *mut u8).offset(-(offset_of!($container, $field) as isize)) as *mut $container
    }
}

pub unsafe fn wl_signal_add(signal: *mut wl_signal, listener: *mut wl_listener) {
    wl_list_insert((*signal).listener_list.prev, &mut (*listener).link);
}
