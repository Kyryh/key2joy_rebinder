#![cfg(windows)]
use std::num::NonZero;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{process, thread};

use eframe::{App, CreationContext, egui};
use egui_winit::winit::raw_window_handle::{HasWindowHandle as _, RawWindowHandle};
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, ListenerOptions, NameType as _, Stream, ToFsName, ToNsName,
    traits::{ListenerExt as _, Stream as _},
};
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};

use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging};

mod winzozz;
use windows::core::HRESULT;
use winzozz::*;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        centered: true,
        ..Default::default()
    };
    eframe::run_native("Key2Joy Rebinder", options, Box::new(MyApp::app_creator))
}

struct MyApp {
    window_handle: NonZero<isize>,
    available_processes: Arc<Mutex<AvailableProcesses>>,
    _tray_icon: TrayIcon,
}

fn set_visibility(window_handle: NonZero<isize>, visibility: bool) {
    let show = if visibility {
        WindowsAndMessaging::SW_SHOWDEFAULT
    } else {
        WindowsAndMessaging::SW_HIDE
    };
    unsafe {
        let res = WindowsAndMessaging::ShowWindow(HWND(window_handle.get() as _), show).ok();
        if let Err(err) = res
            && err.code() != HRESULT(0)
        {
            panic!("{err:?}");
        }
    }
}

impl MyApp {
    fn app_creator(
        c: &CreationContext,
    ) -> Result<Box<dyn App>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Box::new(Self::new(c)))
    }

    fn new(c: &CreationContext) -> Self {
        let window_handle = match c.window_handle().unwrap().as_raw().clone() {
            RawWindowHandle::Win32(win32_window) => win32_window.hwnd,
            _ => unreachable!(),
        };

        let socket_name = {
            if GenericNamespaced::is_supported() {
                "key2joy_rebinder.sock".to_ns_name::<GenericNamespaced>()
            } else {
                "/tmp/key2joy_rebinder.sock".to_fs_name::<GenericFilePath>()
            }
        }
        .unwrap();

        if let Ok(_) = Stream::connect(socket_name.clone()) {
            process::exit(0);
        }

        let ctx = c.egui_ctx.clone();
        thread::spawn(move || {
            let listener = ListenerOptions::new()
                .name(socket_name)
                .create_sync()
                .unwrap();
            for _ in listener.incoming() {
                set_visibility(window_handle, true);
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        });

        MenuEvent::set_event_handler(Some(move |evt: MenuEvent| match evt.id.as_ref() {
            "OPEN" => {
                #[cfg(target_os = "windows")]
                set_visibility(window_handle, true);
            }
            "QUIT" => {
                process::exit(0);
            }
            _ => {}
        }));

        let available_processes = Arc::<Mutex<AvailableProcesses>>::default();
        let available_processes_clone = available_processes.clone();
        thread::spawn(move || {
            loop {
                let processes = get_available_processes();
                *available_processes_clone.lock().unwrap() = processes;
                thread::sleep(Duration::from_secs(5));
            }
        });

        Self {
            window_handle,
            available_processes,
            _tray_icon: TrayIconBuilder::new()
                .with_tooltip("Key2Joy Rebinder")
                .with_menu(Box::new(
                    Menu::with_items(&[
                        &MenuItem::with_id("OPEN", "Open", true, None),
                        &MenuItem::with_id("QUIT", "Quit", true, None),
                    ])
                    .unwrap(),
                ))
                // .with_icon(icon)
                .build()
                .unwrap(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            set_visibility(self.window_handle, false);
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns_const(|[ui_left, ui_right]| {
                egui::Frame::new()
                    .fill(egui::Color32::BLACK)
                    .corner_radius(5)
                    .inner_margin(2)
                    .show(ui_left, |ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            ui.label("PROCESSES:");
                        });
                        for process in self.available_processes.lock().unwrap().iter() {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                if ui.button("➕").clicked() {
                                    // TODO
                                }
                                ui.label(&process.0);
                            });
                        }
                    });
                egui::Frame::new()
                    .fill(egui::Color32::BLACK)
                    .corner_radius(5)
                    .inner_margin(2)
                    .show(ui_right, |ui| {
                        // TODO
                    });
            });
        });
    }
}
