use chip8sys::chip8::Chip8Sys;
use chip8sys::chip8error::Chip8Error;
use egui::special_emojis;
use egui::text::LayoutJob;
use egui::{Color32, Key};
use egui_extras::{Column, TableBuilder};
use rodio::mixer::Mixer;
use rodio::source::{SineWave, Source};

use crate::about::About;

// if we add new fields, give them default values when deserializing old state
pub struct Chip8App {
    chip8: Chip8Sys,
    zoom: f32,
    background_color: Color32,
    pixel_color: Color32,
    key_map: [Key; 16],
    sink: rodio::Sink,
    compute_info: ConfigWindow,
    screen_config: ConfigWindow,
    about: ConfigWindow,
    control_flow: ConfigWindow,
    run: bool,
    single_step: bool,
    rom_path: String,
}

impl Default for Chip8App {
    fn default() -> Self {
        // Setup and Handle Sound
        // let sink = rodio::Sink::connect_new(&stream_handle.mixer());

        Self {
            // Example stuff:
            chip8: Chip8Sys::new_chip_8(),
            zoom: 15.0,
            background_color: Color32::BLACK,
            pixel_color: Color32::GREEN,
            key_map: [
                Key::X,
                Key::Num1,
                Key::Num2,
                Key::Num3,
                Key::Q,
                Key::W,
                Key::E,
                Key::A,
                Key::S,
                Key::D,
                Key::Z,
                Key::C,
                Key::Num4,
                Key::R,
                Key::F,
                Key::V,
            ],
            sink: rodio::Sink::new().0,
            screen_config: ConfigWindow {
                name: String::from("Screen Config"),
                show: false,
            },
            compute_info: ConfigWindow {
                name: String::from("Compute Info"),
                show: true,
            },
            about: ConfigWindow {
                name: String::from("About Chip-8"),
                // TODO: Make this true long term
                show: false,
            },
            control_flow: ConfigWindow {
                name: String::from("Control Flow"),
                show: true,
            },
            run: true,
            single_step: false,
            rom_path: String::new(),
        }
    }
}

impl Chip8App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>, mixer: Result<&Mixer, String>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let mut result: Chip8App = Default::default();

        // Setup Sound
        // If I send a mixer in use it, otherwise ignore it
        // NOTE: WILL Cause problems if someone requests sound via chip8 and it's not there.
        match mixer {
            Ok(m) => result.sink = rodio::Sink::connect_new(m),
            Err(_) => (),
        }

        // Load Chip-8 Roms
        // result.rom_path = "roms/1-chip8-logo.ch8".to_string();
        // result.chip8.load_rom("roms/2-ibm-logo.ch8".to_string());
        // result.chip8.load_rom("roms/3-corax+.ch8".to_string());
        // result.rom_path = "roms/5-quirks.ch8".to_string();
        // When running quirks rom hardcode this memory spot to auto run Chip-8
        // result.chip8.memory[0x1FF] = 1;
        result.rom_path = "roms/walking_man.ch8".to_string();
        // result.rom_path = "../roms/7-beep.ch8".to_string();

        result.chip8.load_rom(&result.rom_path);
        // result.chip8.load_chip8_logo();
        // result.chip8.load_sound_test();
        //

        /*
        println!("chip8-logo");
        println!("");
        println!("{:?}", result.chip8.memory);
        println!("");
        // */
        result
    }
}

impl eframe::App for Chip8App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // If we're restarting the chip-8 then reinitialize it with quirks
        // Scan the keys and if they're pressed tell the Chip-8
        // I think I'm missing an edge case quirk thing where chip-8 only acts if a key is released
        // But that might be beyond my scope of interest for this project
        ctx.input(|i| {
            for (n, k) in self.key_map.iter().enumerate() {
                self.chip8.keys[n] = i.key_pressed(*k);
            }
        });

        // Handle Sound
        if self.chip8.is_playing_sound {
            self.sink.append(SineWave::new(440.0).repeat_infinite());
        } else {
            self.sink.stop();
        }

        // TODO: Not sure how I want to handle all these yet...
        // maybe log them in their own window?
        if self.run | self.single_step {
            match self.chip8.run() {
                Ok(_) => (),
                Err(e) => match e {
                    // if the N of 0xN___ is invalid it will return this and the N provided
                    Chip8Error::InvalidFirstByte(_) => (),
                    // If the X register should be <= 0xF
                    Chip8Error::InvalidRegisterX(_) => (),
                    // if the N in 0x8XYN is invalid it will return this and the N provided
                    Chip8Error::Invalid0x8XYN(_) => (),
                    // if the N in 0x8XYN is invalid it will return this and the N provided
                    Chip8Error::Invalid0xENNN(_, _) => (),
                    // if the N in 0x8XYN is invalid it will return this and the N provided
                    Chip8Error::Invalid0xFNNN(_, _) => (),
                    // If the register we're waiting for is somehow > 0xF
                    Chip8Error::InvalidWaitRegister(_) => (),
                    Chip8Error::IssueGeneratingRandomNum(_) => (),
                },
            }
            if self.single_step {
                self.single_step = false;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Chip-8 Display".to_string());
            let painter = ui.painter();

            let width = self.zoom;
            let mut row = 0.0;
            let mut col = 0.0;
            let x_off = 50.0;
            let y_off = 45.0;
            let col_count = 8;

            for (n, px) in self.chip8.frame_buffer.iter().enumerate() {
                if n % col_count == 0 {
                    row += width;
                    col = 0.0;
                }
                let mut bit_stream: Vec<bool> = Vec::new();

                for b in 0..8 {
                    bit_stream.push(((px << b) & 0b1000_0000) == 0b1000_0000);
                }
                for cell in bit_stream {
                    let x_start = x_off + (col * width);
                    let y_start = y_off + row;
                    let color: Color32;
                    if cell {
                        color = self.pixel_color;
                    } else {
                        color = self.background_color;
                    };
                    painter.rect_filled(
                        egui::Rect {
                            min: egui::Pos2 {
                                x: x_start,
                                y: y_start,
                            },
                            max: egui::Pos2 {
                                x: x_start + width,
                                y: y_start + width,
                            },
                        },
                        0.0,
                        color,
                    );
                    col += 1.0;
                }
            }
            ctx.request_repaint();
        });

        egui::SidePanel::right("Config Toggle").show(ctx, |ui| {
            ui.heading("Chip-8 Toolbox");
            ui.separator();
            ui.toggle_value(&mut self.about.show, self.about.name.clone());
            ui.toggle_value(&mut self.compute_info.show, self.compute_info.name.clone());
            ui.toggle_value(
                &mut self.screen_config.show,
                self.screen_config.name.clone(),
            );
            ui.toggle_value(&mut self.control_flow.show, self.control_flow.name.clone());
            let is_web = cfg!(target_arch = "wasm32");
            if !is_web {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        });

        egui::Window::new(self.compute_info.name.clone())
            .open(&mut self.compute_info.show)
            .show(ctx, |ui| {
                ctx.set_pixels_per_point(2.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // The central panel the region left after adding TopPanel's and SidePanel's
                    ui.horizontal(|ui| {
                        ui.label(format!("Program Counter: {}", &self.chip8.program_counter));
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("Register I: {}", &self.chip8.register_i));
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("Stack Pointer: {}", &self.chip8.stack_pointer));
                    });
                    ui.separator();
                    let available_height = ui.available_height();
                    let table = TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto())
                        .column(Column::auto())
                        .column(Column::auto())
                        .column(Column::auto())
                        .min_scrolled_height(0.0)
                        .max_scroll_height(available_height);
                    // table = table.sense(egui::Sense::click());

                    table
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("Index");
                            });
                            header.col(|ui| {
                                ui.strong("Register Value");
                            });
                            header.col(|ui| {
                                ui.strong("Stack Value");
                            });
                            header.col(|ui| {
                                ui.strong("Keys");
                            });
                        })
                        .body(|mut body| {
                            for row_index in 0..self.chip8.register.len() {
                                body.row(30.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(format!("0x{:X}", row_index));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "0x{:02X}",
                                            self.chip8.register[row_index]
                                        ));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("0x{:04X}", self.chip8.stack[row_index]));
                                    });
                                    row.col(|ui| {
                                        if self.chip8.keys[row_index] {
                                            ui.label("Pressed");
                                        } else {
                                            ui.label("");
                                        }
                                    });
                                });
                            }
                        });
                    // self.table(ui);
                    ui.separator();
                });
            });
        egui::Window::new(self.screen_config.name.clone())
            .open(&mut self.screen_config.show)
            .show(ctx, |ui| {
                ctx.set_pixels_per_point(2.0);
                ui.add(egui::Slider::new(&mut self.zoom, 0.0..=25.0).text("Zoom: "));
                ui.label("Pixel: ");
                ui.color_edit_button_srgba(&mut self.pixel_color);
                ui.label("Background: ");
                ui.color_edit_button_srgba(&mut self.background_color);
                egui::widgets::global_theme_preference_buttons(ui);
            });

        egui::Window::new(self.about.name.clone())
            .open(&mut self.about.show)
            .show(ctx, |ui| {
                ui.heading("Chip-8 Emulator");
                ui.label("By Nicholas Licalsi");
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink(true)
                    .show(ui, |ui| {
                        let mut job = LayoutJob::single_section(
                            crate::about::About::about(),
                            egui::TextFormat {
                                ..Default::default()
                            },
                        );
                        job.wrap = egui::text::TextWrapping {
                            max_rows: 10,
                            ..Default::default()
                        };

                        // NOTE: `Label` overrides some of the wrapping settings, e.g. wrap width
                        ui.label(job);
                    });
                ui.separator();
                ui.label(format!("{} Source Code", special_emojis::GITHUB));
                ui.hyperlink("https://github.com/licalsinj/chip-8");
            });
        egui::Window::new(self.control_flow.name.clone())
            .open(&mut self.control_flow.show)
            .show(ctx, |ui| {
                ui.heading("Chip-8 Control Flow");
                ui.label("Pause or run the emulator. When paused you can use Single Step to walk through one command at a time.");
                ui.separator();
                egui::Grid::new("control_flow_controls").show(ui,|ui|{
                    let state = if self.run { "Pause" } else { "Run" };
                    ui.toggle_value(&mut self.run, state);
                    if ui.add_enabled(!self.run, egui::Button::new("Single Step")).clicked() {
                        self.single_step = true;
                    }
                    if ui.button("Restart").clicked() {
                        // TODO: Take into account quirks
                        self.chip8 = Chip8Sys::new_chip_8();
                        // TODO: do this better right now I want to know if wasm works
                        self.chip8.load_rom(&self.rom_path);
                        // self.chip8.memory[0x1FF] = 1;
                        // self.chip8.load_chip8_logo();
                        // self.chip8.load_sound_test();
                    }
                    ui.end_row();
                });
                ui.separator();
                egui::Grid::new("instruction_output").show(ui, |ui| {
                    ui.label(format!("Program Counter: 0x{:04X}", self.chip8.program_counter));
                    ui.end_row();

                    let prev_instruction_high = self.chip8.memory[self.chip8.program_counter as usize - 2];
                    let prev_instruction_low = self.chip8.memory[self.chip8.program_counter as usize - 1];
                    ui.label(format!("Previous Instruction: 0x{:02X}{:02X}", prev_instruction_high, prev_instruction_low));
                    ui.label(About::chip_8_decode(prev_instruction_high, prev_instruction_low));
                    ui.end_row();

                    let next_instruction_high = self.chip8.memory[self.chip8.program_counter as usize];
                    let next_instruction_low = self.chip8.memory[self.chip8.program_counter as usize + 1];
                    ui.label(format!("Next Instruction: 0x{:02X}{:02X}", next_instruction_high, next_instruction_low));
                    ui.label(About::chip_8_decode(next_instruction_high,next_instruction_low));
                    ui.end_row();
                });
            });
    }
}

struct ConfigWindow {
    name: String,
    show: bool,
}
