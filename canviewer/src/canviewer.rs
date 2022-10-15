use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, fs::File, io::Read, iter::Map};

use backend::{load_dbc_from_bytes, parse_signal};
use ecu_diagnostics::{hardware::{Hardware, HardwareScanner, socketcan::{SocketCanDevice, SocketCanCanChannel, SocketCanScanner}}, channel::{CanChannel, CanFrame, Packet}};
use eframe::{egui::*, epaint::{mutex::RwLock, ahash::{HashMap, HashMapExt}}};
use egui_extras::*;
use std::fmt::Write;

pub struct CanViewer {
    dbc: Option<backend::TreeDbc>,
    dbc_name: String,
    iface_name: String,
    is_reading: Arc<AtomicBool>,
    load_error: Option<String>,
    frames: Arc<RwLock<HashMap<u32, CanFrame>>>,
    frames_previous: HashMap<u32, CanFrame>,
    open_frames: Vec<(usize, usize)>
}


impl CanViewer {
    pub fn new(iface_name: String, dbc_path: Option<String>) -> ecu_diagnostics::hardware::HardwareResult<Self> {
        let scanner = SocketCanScanner::new();
        let can_hw = scanner.open_device_by_name(&iface_name)?;
        let mut can_channel = Hardware::create_can_channel(can_hw)?;
        can_channel.open();
        let is_reading = Arc::new(AtomicBool::new(true));
        let frame_list = Arc::new(RwLock::new(HashMap::new()));
        let is_reading_c = is_reading.clone();
        let frame_list_c = frame_list.clone();

        std::thread::spawn(move|| {
            loop {
                if is_reading_c.load(Ordering::Relaxed) {
                    match can_channel.read_packets(100, 10) {
                        Ok(res) => {
                            let mut lock = frame_list_c.write();
                            for f in res {
                                lock.insert(f.get_address(), f);
                            }
                        }
                        Err(e) => {
                            eprintln!("Read error: {}", e);
                        }
                    }
                } else {
                    can_channel.clear_rx_buffer();
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        let mut dbc = None;
        let mut load_error = None;
        if let Some(path) = &dbc_path {
            if let Ok(dbc_bytes) = File::open(path)
                .and_then(|mut f| {
                    let mut buf = Vec::new();
                    f.read_to_end(&mut buf)?;
                    Ok(buf)
            }) {
                match load_dbc_from_bytes(&dbc_bytes) {
                    Ok(d) => {
                        dbc = Some(d)
                    },
                    Err(e) => {
                        load_error = Some(e.to_string())
                    }
                }
            }
        }

        Ok(Self {
            dbc,
            dbc_name: dbc_path.unwrap_or_default(),
            iface_name,
            is_reading,
            load_error,
            frames: frame_list,
            frames_previous: HashMap::new(),
            open_frames: Vec::new()
        })

    }
}

impl eframe::App for CanViewer {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        SidePanel::left("MainBar").show(ctx, |ui| {
            ui.heading("CanViewerRS");
            ui.separator();
            ui.label(format!("Connected to {}", self.iface_name));
            
            if ui.button("Pause/Play").clicked() {
                let new_state = !self.is_reading.load(Ordering::Relaxed);
                self.is_reading.store(new_state, Ordering::Relaxed);
            }

            ui.heading("DBC Explorer");
            ui.separator();
            if let Some(dbc) = &self.dbc {
                ScrollArea::new([true, true]).show(ui, |scroll| {
                    scroll.collapsing(&self.dbc_name, |dbc_content| {
                        for (ecu_idx, ecu) in dbc.ecus.iter().enumerate() {
                            dbc_content.collapsing(format!("ECU {}", ecu.name), |ecu_ui| {
                                for (msg_idx, msg) in ecu.messages.iter().enumerate() {
                                    ecu_ui.collapsing(format!("Msg {} (0x{:04X})", msg.name, msg.id), |msg_ui| {

                                        if self.open_frames.contains(&(ecu_idx, msg_idx)) {
                                            if msg_ui.button("Hide Frame").clicked() {
                                                self.open_frames.retain(|(e, m)| *e != ecu_idx && *m != msg_idx)
                                            }
                                        } else {
                                            if msg_ui.button("Show Frame").clicked() {
                                                self.open_frames.push((ecu_idx, msg_idx));
                                            }
                                        }

                                        for signal in &msg.signals {
                                            let s_r = msg_ui.label(&signal.name);
                                            if let Some(cmt) = &signal.comment {
                                                s_r.on_hover_text(cmt);
                                            }
                                        }
                                    });
                                }
                            });
                        }
                    });
                });


            } else {
                ui.label("No DBC loaded");
                if let Some(err) = &self.load_error {
                    ui.label(format!("DBC Load error: {}", err));
                }
            }
        });

        // Status bottom bar
        TopBottomPanel::bottom("Statusbar").show(ctx, |ui| {
            widgets::global_dark_light_mode_buttons(ui);
        });

        CentralPanel::default().show(ctx, |cui| {
            // Now show all the CAN Frames!
            if let Some(dbc) = &self.dbc {
                for (ecu_idx, msg_idx) in &self.open_frames {
                    let msg = &dbc.ecus[*ecu_idx].messages[*msg_idx];
                    containers::Window::new(format!("Frame {} (ID 0x{:04X})", msg.name, msg.id)).show(cui.ctx(), |ui| {
                        if let Some(cf) = self.frames.read().get(&msg.id) {
                            ui.label(format!("{:02X?}", cf.get_data()));
                            

                            let mut table = TableBuilder::new(ui)
                                .striped(true)
                                .scroll(true)
                                .clip(false)
                                .cell_layout(Layout::left_to_right(Align::Center).with_cross_align(Align::Center))
                                .column(Size::initial(60.0).at_least(60.0)) // Value name
                                .column(Size::initial(400.0).at_least(500.0)); // Value

                            table.header(15.0, |mut header| {
                                header.col(|u| {u.label("Signal name");});
                                header.col(|u| {u.label("Value");});
                            }).body(|body| {
                                body.rows(18.0, msg.signals.len(), |row_id, mut row| {
                                    let signal = &msg.signals[row_id];
                                    row.col(|x| {
                                        let r = x.label(&signal.name);
                                        if let Some(cmt) = &signal.comment {
                                            r.on_hover_text(cmt);
                                        }
                                    });
                                    row.col(|x| {
                                        match parse_signal(signal, cf.get_data()) {
                                            Ok(s) => {
                                                x.label(s.to_string());
                                            },
                                            Err(e) => {
                                                x.label(RichText::new(format!("{:?}", e)).color(Color32::RED));
                                            }
                                        }
                                    });

                                })
                            });
                        } else {
                            ui.label("No CAN data for this frame.");
                        }
                    });
                }
            }

            containers::Window::new("Frame viewer").show(cui.ctx(), |ui| {
                let mut table = TableBuilder::new(ui)
                                .striped(true)
                                .scroll(true)
                                .clip(false)
                                .cell_layout(Layout::left_to_right(Align::Center).with_cross_align(Align::Center))
                                .column(Size::initial(60.0).at_least(60.0)) // CAN ID
                                .column(Size::initial(30.0).at_least(30.0)) //1st byte
                                .column(Size::initial(30.0).at_least(30.0)) //2nd byte
                                .column(Size::initial(30.0).at_least(30.0)) //3rd byte
                                .column(Size::initial(30.0).at_least(30.0)) //4th byte
                                .column(Size::initial(30.0).at_least(30.0)) //5th byte
                                .column(Size::initial(30.0).at_least(30.0)) //6th byte
                                .column(Size::initial(30.0).at_least(30.0)) //7th byte
                                .column(Size::initial(30.0).at_least(30.0)) //8th byte
                                .column(Size::initial(100.0).at_least(100.0)); // ASCII

                            table.header(15.0, |mut header| {
                                header.col(|u| {u.label("CAN ID");});
                                header.col(|u| {u.label("");});
                                header.col(|u| {u.label("");});
                                header.col(|u| {u.label("");});
                                header.col(|u| {u.label("");});
                                header.col(|u| {u.label("");});
                                header.col(|u| {u.label("");});
                                header.col(|u| {u.label("");});
                                header.col(|u| {u.label("");});
                                header.col(|u| {u.label("ASCII");});
                            }).body(|body| {
                                let mut map_clone: Vec<CanFrame> = self.frames.read().values().cloned().collect();
                                map_clone.sort_by(|c, n| { c.get_address().cmp(&n.get_address()) });
                                body.rows(18.0, map_clone.len(), |r, mut row| {
                                    let frame = map_clone[r];
                                    row.col(|u| {u.label(format!("0x{:04X}", frame.get_address()));});

                                    let mut ascii = String::new();
                                    for idx in 0..8 {
                                        match frame.get_data().get(idx) {
                                            Some(byte) => {
                                                row.col(|u| {
                                                    let mut l = RichText::new(format!("{:02X}", byte));
                                                    if let Some(of) = self.frames_previous.get(&frame.get_address()) {
                                                        if of.get_data()[idx] > *byte {
                                                            l = l.color(Color32::RED);
                                                        } else if of.get_data()[idx] < *byte {
                                                            l = l.color(Color32::BLUE);
                                                        }
                                                    }
                                                    u.label(l);
                                                });
                                                if byte.is_ascii_graphic() {
                                                    write!(ascii, "{}", String::from_utf8_lossy(&[*byte]));
                                                } else {
                                                    ascii.push_str(".");
                                                }
                                            },
                                            None => {
                                                row.col(|x| {});
                                            }
                                        }
                                    }
                                    self.frames_previous.insert(frame.get_address(), frame);
                                    // ASCII row
                                    row.col(|x| {x.label(ascii);});
                                })
                            });

            });
            ctx.request_repaint();
        });

    }
}