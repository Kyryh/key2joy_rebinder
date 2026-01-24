use std::{
    ffi::{CStr, OsStr, c_void},
    mem,
    os::windows::ffi::OsStrExt,
};

use retour::StaticDetour;
use windows::{
    Win32::{
        Foundation::HINSTANCE,
        System::{
            Console::AllocConsole,
            LibraryLoader::{GetModuleHandleW, GetProcAddress},
            SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
        },
        UI::Input::{KeyboardAndMouse::GetKeyState, XboxController::XINPUT_STATE},
    },
    core::{PCSTR, PCWSTR},
};

mod config;
use config::{CONFIG, GamepadInput, Vec2};

/// Returns 1 if pressed, 0 if not pressed
fn key_pressed(keycode: i32) -> u16 {
    unsafe { GetKeyState(keycode) as u16 >> 15 }
}

pub static HOOK: StaticDetour<unsafe extern "system" fn(u32, *mut XINPUT_STATE) -> u32> = {
    #[inline(never)]
    unsafe extern "system" fn ffi_detour(dwuserindex: u32, pstate: *mut XINPUT_STATE) -> u32 {
        (HOOK.__detour())(dwuserindex, pstate)
    }
    StaticDetour::__new(ffi_detour)
};

pub fn xinput_get_state_hook(dw_user_index: u32, pstate: *mut XINPUT_STATE) -> u32 {
    let result = unsafe { HOOK.call(dw_user_index, pstate) };
    if dw_user_index == 0
        && let Some(pstate) = unsafe { pstate.as_mut() }
    {
        let mut left_stick = Vec2::new(0., 0.);
        let mut right_stick = Vec2::new(0., 0.);
        for (gamepad_input, keycode) in &CONFIG.keys_to_gamepad_map {
            if key_pressed(*keycode) != 0 {
                match gamepad_input {
                    GamepadInput::Button(button) => pstate.Gamepad.wButtons.0 |= button,
                    GamepadInput::LeftTrigger => pstate.Gamepad.bLeftTrigger = u8::MAX,
                    GamepadInput::RightTrigger => pstate.Gamepad.bRightTrigger = u8::MAX,
                    GamepadInput::LeftStick(direction) => left_stick += direction,
                    GamepadInput::RightStick(direction) => right_stick += direction,
                }
            }
        }
        if left_stick != Vec2::ZERO {
            left_stick.normalize();
            left_stick *= i16::MAX as f32;
            pstate.Gamepad.sThumbLX = left_stick.x as i16;
            pstate.Gamepad.sThumbLY = left_stick.y as i16;
        }
        if right_stick != Vec2::ZERO {
            right_stick.normalize();
            right_stick *= i16::MAX as f32;
            pstate.Gamepad.sThumbRX = right_stick.x as i16;
            pstate.Gamepad.sThumbRY = right_stick.y as i16;
        }
    }
    result
}

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_: HINSTANCE, fdw_reason: u32, _: *const c_void) -> bool {
    match fdw_reason {
        DLL_PROCESS_ATTACH => unsafe {
            if let Some(xinput_get_state_address) = find_xinput_get_state_address() {
                if CONFIG.enable_console {
                    AllocConsole().unwrap();
                }
                println!("Detected bindings:");
                for (gamepad_input, keycode) in &CONFIG.keys_to_gamepad_map {
                    println!("- {keycode} => {gamepad_input:?}");
                }
                HOOK.initialize(
                    mem::transmute(xinput_get_state_address),
                    xinput_get_state_hook,
                )
                .unwrap()
                .enable()
                .unwrap();
            }
        },
        DLL_PROCESS_DETACH => unsafe {
            if HOOK.is_enabled() {
                HOOK.disable().unwrap()
            }
        },
        _ => {}
    }
    true
}

fn find_xinput_get_state_address() -> Option<usize> {
    for lib_name in [
        "xinput1_4.dll\0",
        "xinput1_3.dll\0",
        "xinput1_2.dll\0",
        "xinput1_1.dll\0",
        "xinput9_1_0.dll\0",
    ] {
        if let Some(address) = get_module_symbol_address(lib_name, c"XInputGetState") {
            return Some(address);
        }
    }

    None
}

fn get_module_symbol_address(module: &str, symbol: &CStr) -> Option<usize> {
    let module: Vec<_> = OsStr::new(module).encode_wide().collect();
    unsafe {
        let handle = GetModuleHandleW(PCWSTR(module.as_ptr())).ok()?;
        GetProcAddress(handle, PCSTR(symbol.as_ptr().cast())).map(|addr| addr as usize)
    }
}
