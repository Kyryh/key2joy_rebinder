use std::ffi::c_void;

use windows::Win32::{
    Foundation::HINSTANCE,
    System::{Console::AllocConsole, SystemServices::DLL_PROCESS_ATTACH},
    UI::Input::{KeyboardAndMouse::GetKeyState, XboxController::XINPUT_GAMEPAD},
};

/// Returns 1 if pressed, 0 if not pressed
fn key_pressed(key: char) -> u16 {
    unsafe { GetKeyState(key as i32) as u16 >> 15 }
}

pub fn inject_keyboard_input(dw_user_index: u32, p_state: &mut XINPUT_GAMEPAD) {
    if dw_user_index == 0 {
        p_state.wButtons.0 |= 0b0
            | key_pressed('W') << 0
            | key_pressed('S') << 1
            | key_pressed('A') << 2
            | key_pressed('D') << 3;
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_: HINSTANCE, fdw_reason: u32, _: *const c_void) -> bool {
    if fdw_reason == DLL_PROCESS_ATTACH {
        unsafe {
            AllocConsole().unwrap();
        }
    }
    true
}
