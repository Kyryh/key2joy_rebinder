use std::ffi::c_void;

use retour::StaticDetour;
use windows::Win32::{
    Foundation::HINSTANCE,
    System::{
        Console::AllocConsole,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    },
    UI::Input::{KeyboardAndMouse::GetKeyState, XboxController::XINPUT_STATE},
};

/// Returns 1 if pressed, 0 if not pressed
fn key_pressed(key: char) -> u16 {
    unsafe { GetKeyState(key as i32) as u16 >> 15 }
}

pub static HOOK: StaticDetour<unsafe extern "system" fn(u32, *mut XINPUT_STATE) -> u32> = {
    #[inline(never)]
    unsafe extern "system" fn ffi_detour(dwuserindex: u32, pstate: *mut XINPUT_STATE) -> u32 {
        (HOOK.__detour())(dwuserindex, pstate)
    }
    StaticDetour::__new(ffi_detour)
};

windows::core::link!("xinput1_4.dll" "system" fn XInputGetState(dwuserindex: u32, pstate: *mut XINPUT_STATE) -> u32);

pub fn xinput_get_state_hook(dw_user_index: u32, pstate: *mut XINPUT_STATE) -> u32 {
    let result = unsafe { HOOK.call(dw_user_index, pstate) };
    if dw_user_index == 0
        && let Some(pstate) = unsafe { pstate.as_mut() }
    {
        pstate.Gamepad.wButtons.0 |= 0b0
            | key_pressed('W') << 0
            | key_pressed('S') << 1
            | key_pressed('A') << 2
            | key_pressed('D') << 3;
    }
    result
}

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_: HINSTANCE, fdw_reason: u32, _: *const c_void) -> bool {
    match fdw_reason {
        DLL_PROCESS_ATTACH => unsafe {
            AllocConsole().unwrap();
            HOOK.initialize(XInputGetState, xinput_get_state_hook)
                .unwrap()
                .enable()
                .unwrap();
        },
        DLL_PROCESS_DETACH => unsafe { HOOK.disable().unwrap() },
        _ => {}
    }
    true
}
