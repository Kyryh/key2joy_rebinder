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
            HOOK.initialize(
                mem::transmute(find_xinput_get_state_address().unwrap()),
                xinput_get_state_hook,
            )
            .unwrap()
            .enable()
            .unwrap();
        },
        DLL_PROCESS_DETACH => unsafe { HOOK.disable().unwrap() },
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
