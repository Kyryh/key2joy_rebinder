use eframe::{App, CreationContext, egui};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native("Key2Joy Rebinder", options, Box::new(MyApp::app_creator))
}

struct MyApp {}

impl MyApp {
    fn app_creator(
        c: &CreationContext,
    ) -> Result<Box<dyn App>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Box::new(Self::new(c)))
    }

    fn new(c: &CreationContext) -> Self {
        Self {}
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {});
    }
}
