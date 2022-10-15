use canviewer::CanViewer;
use clap::{Parser};
mod canviewer;
mod error_page;
use eframe::{NativeOptions, Renderer, IconData, epaint::Vec2};

use ecu_diagnostics::*;
use error_page::ErrorPage;

#[derive(Debug, Parser, Clone)]
pub struct CanViewerSettings {
    /// Socket CAN Interface name to connect to
    socketcan_iface: String,
    /// Optional DBC File to load
    dbc_file: Option<String>
}

fn main() {
    let args = CanViewerSettings::parse();

    let icon = image::load_from_memory(include_bytes!("../logo.png")).unwrap().to_rgba8();
    let (icon_w, icon_h) = icon.dimensions();

    #[cfg(unix)]
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    let mut native_options = NativeOptions::default();
    native_options.icon_data = Some(IconData{
        rgba: icon.into_raw(),
        width: icon_w,
        height: icon_h,
    });
    native_options.initial_window_size = Some(Vec2::new(1280.0, 720.0));

    #[cfg(windows)]
    {
        native_options.renderer = Renderer::Wgpu;
    }
    let c = args.socketcan_iface.clone();
    eframe::run_native("CanViewerRS", native_options, Box::new(|cc| {
        match CanViewer::new(c, args.dbc_file) {
            Ok(viewer) => {
                Box::new(viewer)
            },
            Err(e) => {
                Box::new(ErrorPage::new(e.to_string()))
            }
        }
    }));

    
}
