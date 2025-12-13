use std::{ffi::OsStr, iter, mem, os::windows::ffi::OsStrExt, ptr};

use winapi::{
    shared::{
        minwindef::{BYTE, DWORD, HINSTANCE, HMODULE, LPVOID, WORD},
        ntdef::{HANDLE, SHORT},
    },
    um::{
        consoleapi::AllocConsole,
        libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryExW},
        winnt::DLL_PROCESS_ATTACH,
    },
};

use crate::inject_keyboard_input;

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_: HINSTANCE, fdw_reason: DWORD, _: LPVOID) -> bool {
    if fdw_reason == DLL_PROCESS_ATTACH {
        unsafe {
            AllocConsole();
        }
    }
    true
}

static mut XINPUT_MODULE: HMODULE = std::ptr::null_mut();

static mut FN_GET_CAPABILITIES: Option<fn(DWORD, &mut XInputState)> = None;
static mut FN_GET_STATE: Option<fn(DWORD, *mut XInputState) -> DWORD> = None;
static mut FN_SET_STATE: Option<fn(DWORD, &mut XInputVibration)> = None;

#[allow(unsafe_op_in_unsafe_fn)]
#[allow(static_mut_refs)]
unsafe fn load_fns() -> u64 {
    if XINPUT_MODULE.is_null() {
        let lib_name: Vec<_> = OsStr::new("XInput1_4.dll")
            .encode_wide()
            .chain(iter::once(0))
            .collect();

        XINPUT_MODULE = LoadLibraryExW(lib_name.as_ptr(), 0 as HANDLE, 0x800);
        if XINPUT_MODULE.is_null() {
            return 0x7e;
        }

        FN_GET_CAPABILITIES = mem::transmute(GetProcAddress(
            XINPUT_MODULE,
            b"XInputGetCapabilities\0".as_ptr() as *const _,
        ));
        FN_GET_STATE = mem::transmute(GetProcAddress(
            XINPUT_MODULE,
            b"XInputGetState\0".as_ptr() as *const _,
        ));
        FN_SET_STATE = mem::transmute(GetProcAddress(
            XINPUT_MODULE,
            b"XInputSetState\0".as_ptr() as *const _,
        ));
        if FN_GET_CAPABILITIES.is_none() || FN_GET_STATE.is_none() || FN_SET_STATE.is_none() {
            FN_GET_CAPABILITIES = None;
            FN_GET_STATE = None;
            FN_SET_STATE = None;
            FreeLibrary(XINPUT_MODULE);
            XINPUT_MODULE = ptr::null_mut();
            return 0x7f;
        }
    }
    0
}

#[allow(unsafe_op_in_unsafe_fn)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn XInputGetState(
    dw_user_index: DWORD,
    p_state: *mut XInputState,
) -> DWORD {
    load_fns();
    if let Some(fun) = FN_GET_STATE {
        let result = fun(dw_user_index, p_state);

        inject_keyboard_input(dw_user_index, &mut p_state.as_mut().unwrap().gamepad);
        result
    } else {
        0
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn XInputSetState(
    dw_user_index: DWORD,
    p_vibration: &mut XInputVibration,
) {
    load_fns();
    if let Some(fun) = FN_SET_STATE {
        fun(dw_user_index, p_vibration)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct XInputState {
    dw_packet_number: DWORD,
    gamepad: XInputGamepad,
}

#[repr(C)]
#[derive(Debug)]
pub struct XInputGamepad {
    pub w_buttons: WORD,
    pub b_left_trigger: BYTE,
    pub b_right_trigger: BYTE,
    pub s_thumb_lx: SHORT,
    pub s_thumb_ly: SHORT,
    pub s_thumb_rx: SHORT,
    pub s_thumb_ry: SHORT,
}

#[repr(C)]
#[derive(Debug)]
pub struct XInputVibration {
    w_left_motor_speed: WORD,
    w_right_motor_speed: WORD,
}
