use std::path::PathBuf;

use windows::Win32::{
    Foundation::{HANDLE, HMODULE},
    System::{
        ProcessStatus::{
            EnumProcessModulesEx, EnumProcesses, GetModuleFileNameExA, GetProcessImageFileNameA,
            LIST_MODULES_ALL,
        },
        Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
};

pub type AvailableProcesses = Vec<(String, u32)>;

pub fn get_available_processes() -> AvailableProcesses {
    let mut process_pids = [0; 1024];
    let len = get_processes(&mut process_pids);

    process_pids[..len]
        .into_iter()
        .filter_map(|pid| Some((pid, pid_to_handle(*pid).ok()?)))
        .filter(|(_, process_handle)| {
            let mut modules = [Default::default(); 1024];
            let Ok(len) = get_process_modules(*process_handle, &mut modules) else {
                return false;
            };
            for module in &modules[..len] {
                let mut buf = [0; 256];
                let path = get_module_path(*process_handle, *module, &mut buf);
                if let Some(name) = path.file_name()
                    && name
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .starts_with("xinput")
                {
                    return true;
                }
            }
            false
        })
        .map(|(pid, p)| {
            let mut buf = [0; 256];
            let path = get_process_path(p, &mut buf);
            (path.file_name().unwrap().to_str().unwrap().to_owned(), *pid)
        })
        .collect::<Vec<_>>()
}

fn get_processes(buf: &mut [u32]) -> usize {
    let mut out = 0;
    match unsafe {
        EnumProcesses(
            buf.as_mut_ptr(),
            (size_of::<u32>() * buf.len()) as u32,
            &raw mut out,
        )
    } {
        Ok(_) => dbg!(out) as usize / size_of::<u32>(),
        Err(_) => 0,
    }
}

fn pid_to_handle(pid: u32) -> windows::core::Result<HANDLE> {
    unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) }
}

fn get_process_modules(
    process_handle: HANDLE,
    buf: &mut [HMODULE],
) -> windows::core::Result<usize> {
    let mut out = 0;
    unsafe {
        EnumProcessModulesEx(
            process_handle,
            buf.as_mut_ptr(),
            (size_of::<HMODULE>() * buf.len()) as u32,
            &raw mut out,
            LIST_MODULES_ALL,
        )
        .map(|_| out as usize / size_of::<HMODULE>())
    }
}

fn get_module_path<'a>(
    process_handle: HANDLE,
    module_handle: HMODULE,
    buf: &'a mut [u8],
) -> PathBuf {
    let len = unsafe { GetModuleFileNameExA(Some(process_handle), Some(module_handle), buf) };
    PathBuf::from(str::from_utf8(&buf[..len as usize]).unwrap())
}

fn get_process_path<'a>(process_handle: HANDLE, buf: &'a mut [u8]) -> PathBuf {
    let len = unsafe { GetProcessImageFileNameA(process_handle, buf) };
    PathBuf::from(str::from_utf8(&buf[..len as usize]).unwrap())
}
