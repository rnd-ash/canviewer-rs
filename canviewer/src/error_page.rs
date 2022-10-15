

pub struct ErrorPage {
    msg: String
}

impl ErrorPage {
    pub fn new(msg: String) -> Self {
        Self {msg}
    }
}

impl eframe::App for ErrorPage {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("Error starting CanViewerRS. {}", self.msg));
        });
    }
}