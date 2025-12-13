use std::{ffi::OsStr, iter, mem, os::windows::ffi::OsStrExt, sync::LazyLock};

use winapi::{
    shared::{
        minwindef::{BYTE, DWORD, HINSTANCE, HMODULE, LPVOID, WORD},
        ntdef::{HANDLE, SHORT},
    },
    um::{
        consoleapi::AllocConsole,
        libloaderapi::{GetProcAddress, LoadLibraryExW},
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

static mut XINPUT_LIB: LazyLock<HMODULE> = LazyLock::new(|| unsafe {
    LoadLibraryExW(
        OsStr::new("XInput1_4.dll")
            .encode_wide()
            .chain(iter::once(0))
            .collect::<Vec<_>>()
            .as_ptr(),
        0 as HANDLE,
        0x800,
    )
});

static FN_GET_STATE: LazyLock<fn(DWORD, &mut XInputState) -> DWORD> = LazyLock::new(|| unsafe {
    mem::transmute(GetProcAddress(
        *XINPUT_LIB,
        b"XInputGetState\0".as_ptr() as *const _,
    ))
});
static FN_SET_STATE: LazyLock<fn(DWORD, &mut XInputVibration) -> DWORD> =
    LazyLock::new(|| unsafe {
        mem::transmute(GetProcAddress(
            *XINPUT_LIB,
            b"XInputSetState\0".as_ptr() as *const _,
        ))
    });

#[allow(unsafe_op_in_unsafe_fn)]
#[unsafe(no_mangle)]
pub extern "system" fn XInputGetState(dw_user_index: DWORD, p_state: &mut XInputState) -> DWORD {
    let result = FN_GET_STATE(dw_user_index, p_state);

    inject_keyboard_input(dw_user_index, &mut p_state.gamepad);
    result
}

#[allow(unsafe_op_in_unsafe_fn)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn XInputSetState(
    dw_user_index: DWORD,
    p_vibration: &mut XInputVibration,
) -> DWORD {
    FN_SET_STATE(dw_user_index, p_vibration)
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
