use eframe::egui;
use eframe::epaint::Color32;

use crate::audio::piano_test;
use crate::audio::note::MidiNote;
use crate::audio::play::play;
use crate::audio::instruments::InstrumentKind;

pub fn window() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Harmony",
        options,
        Box::new(|_| Ok(Box::new(MyApp::default()))),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Instrument {
    None,
    Piano,
    Flute,
    Bass,
    Pad,
    Lead,
}

struct Key {
    note: MidiNote,
    color: Color32,
}

struct MyApp {
    name: String,
    enum_instru: Instrument,

    part_piano: [[bool; 255]; 88],
    part_flute: [[bool; 255]; 88],
    part_bass: [[bool; 255]; 88],
    part_pad: [[bool; 255]; 88],
    part_lead: [[bool; 255]; 88],
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "world".to_string(),
            enum_instru: Instrument::Piano,

            part_piano: [[false; 255]; 88],
            part_flute: [[false; 255]; 88],
            part_bass: [[false; 255]; 88],
            part_pad: [[false; 255]; 88],
            part_lead: [[false; 255]; 88],
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.horizontal(|ui| {

                egui::ComboBox::from_id_salt("Files")
                    .selected_text("Files")
                    .show_ui(ui, |ui| {
                        ui.button("Save");
                        ui.button("Import");
                        ui.button("Export");
                    });

                egui::ComboBox::from_id_salt("Instrument")
                    .selected_text(format!("{:?}", self.enum_instru))
                    .show_ui(ui, |ui| {

                        if ui.button("Piano").clicked() {
                            self.enum_instru = Instrument::Piano;
                        }

                        if ui.button("Flute").clicked() {
                            self.enum_instru = Instrument::Flute;
                        }

                        if ui.button("Bass").clicked() {
                            self.enum_instru = Instrument::Bass;
                        }

                        if ui.button("Lead").clicked() {
                            self.enum_instru = Instrument::Lead;
                        }

                        if ui.button("Pad").clicked() {
                            self.enum_instru = Instrument::Pad;
                        }
                    });

                egui::ComboBox::from_id_salt("Effect")
                    .selected_text("Effect")
                    .show_ui(ui, |ui| {
                        ui.button("Reverb");
                        ui.button("?");
                        ui.button("?");
                        ui.button("?");
                    });

            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("sound test :");

                if ui.button("All sound").clicked() {
                    piano_test::run_all_notes();
                }
            });

            ui.separator();

            ui.style_mut().visuals.extreme_bg_color = Color32::GRAY;

            egui::Frame::none()
                .fill(Color32::GRAY)
                .show(ui, |ui| {

                    if self.enum_instru == Instrument::Piano {

                        egui::Grid::new("partition_grid")
                            .num_columns(255)
                            .show(ui, |ui| {

                                let mut nb_key = 0;

                                for i in self.part_piano.iter_mut() {

                                    let nb_key_mod = nb_key % 12;

                                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::WHITE;

                                    if nb_key_mod == 1 || nb_key_mod == 4
                                        || nb_key_mod == 6 || nb_key_mod == 9
                                        || nb_key_mod == 11 {

                                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::BLACK;
                                        ui.button("     ");
                                    }
                                    else {
                                        ui.button("         ");
                                    }

                                    for j in i.iter_mut() {
                                        ui.toggle_value(j, "            ");
                                    }

                                    ui.end_row();
                                    nb_key += 1;
                                }

                            });
                    }

                });

        });
    }
}

pub fn play_note(note: MidiNote, duration_secs: f32)
{
    play(note, InstrumentKind::Piano, duration_secs);
}
