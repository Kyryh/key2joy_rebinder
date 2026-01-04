#![cfg(windows)]

use dll_syringe::{
    Syringe,
    process::{OwnedProcess, Process as _},
};
use std::{
    env,
    fmt::Display,
    io::{self, Read as _, Write as _},
};

fn find_process(name: &str) -> Option<OwnedProcess> {
    OwnedProcess::all()
        .into_iter()
        .filter(|p| match p.base_name() {
            Ok(base_name) => base_name
                .to_ascii_lowercase()
                .to_string_lossy()
                .contains(name),
            Err(_) => false,
        })
        .filter(|p| match p.modules() {
            Ok(modules) => modules.into_iter().any(|m| match m.base_name() {
                Ok(module_name) => module_name
                    .to_ascii_lowercase()
                    .to_string_lossy()
                    .contains("xinput"),
                Err(_) => false,
            }),
            Err(_) => false,
        })
        .next()
}

fn inject(process_name: &str) -> Result<(), Box<dyn Display>> {
    if process_name.is_empty() {
        return Err(Box::new("Process name cannot be empty"));
    }
    let Some(target_process) = find_process(process_name) else {
        return Err(Box::new("No process with XInput loaded found"));
    };

    let dll_path = match env::current_exe() {
        Ok(mut path) => {
            path.pop();
            path.push(
                if target_process.is_x64().map_err(|err| Box::new(err) as _)? {
                    "xinput_injection_x64.dll"
                } else {
                    "xinput_injection_x32.dll"
                },
            );
            path
        }
        Err(err) => return Err(Box::new(err)),
    };

    let syringe = Syringe::for_process(target_process);

    match syringe.inject(&dll_path) {
        Ok(_) => Ok(()),
        Err(err) => Err(Box::new(err)),
    }
}

fn wait_on_exit() -> io::Result<()> {
    println!("Press Enter to continue...");
    let mut stdin = io::stdin();
    stdin.read(&mut []).map(|_| ())
}

fn main() -> io::Result<()> {
    let (process_name, should_wait_on_exit) = match env::args().nth(1) {
        Some(name) => (name, false),
        None => {
            let mut stdout = io::stdout();
            write!(stdout, "Enter process name: ")?;
            stdout.flush()?;
            let stdin = io::stdin();
            let mut buf = String::new();
            stdin.read_line(&mut buf)?;
            (buf, true)
        }
    };
    match inject(&process_name.to_ascii_lowercase().trim_ascii()) {
        Ok(_) => {
            println!("Injected successfully")
        }
        Err(err) => {
            println!("Couldn't inject: {}", err);
        }
    }
    if should_wait_on_exit {
        wait_on_exit()
    } else {
        Ok(())
    }
}
