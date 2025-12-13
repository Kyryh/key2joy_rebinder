mod xinput_exports;

use winapi::{shared::minwindef::DWORD, um::winuser::GetKeyState};

use crate::xinput_exports::XInputGamepad;

/// Returns 1 if pressed, 0 if not pressed
fn key_pressed(key: char) -> u16 {
    unsafe { GetKeyState(key as i32) as u16 >> 15 }
}

pub fn inject_keyboard_input(dw_user_index: DWORD, p_state: &mut XInputGamepad) {
    if dw_user_index == 0 {
        p_state.w_buttons |= key_pressed('W') << 0;
        p_state.w_buttons |= key_pressed('S') << 1;
        p_state.w_buttons |= key_pressed('A') << 2;
        p_state.w_buttons |= key_pressed('D') << 3;
    }
}
