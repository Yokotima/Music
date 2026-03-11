use eframe::egui;
use crate::audio::piano_test;

pub fn window() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Harmony",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::default()))),
    )
}

#[derive(Debug)]
pub enum Instrument
{
    Piano,
    Flute,
    Bass,
    Pad,
    Lead,
}

struct MyApp{
    name: String,
    enum_instru: Instrument,
}


impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "world".to_string(),
            enum_instru: Instrument::Piano,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame:&mut eframe::Frame){
        egui::CentralPanel::default().show(ctx, |ui|{
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("Files")
                    .selected_text(format!("Files"))
                    .show_ui(ui, |ui| {
                        if ui.button("Save").clicked(){

                        }
                        if ui.button("Import").clicked(){

                        }
                        if ui.button("Export").clicked(){

                        }
                    }
                );
                egui::ComboBox::from_id_salt("Instrument")
                    .selected_text(format!("{:?}",self.enum_instru))
                    .show_ui(ui, |ui| {
                        if ui.button("piano").clicked(){
                            self.enum_instru = Instrument::Piano;
                        };
                        if ui.button("flute").clicked(){
                            self.enum_instru = Instrument::Flute;
                        };
                        if ui.button("basse").clicked(){
                            self.enum_instru = Instrument::Bass;
                        };
                        if ui.button("lead").clicked(){
                            self.enum_instru = Instrument::Lead;
                        };
                        if ui.button("pad").clicked(){
                            self.enum_instru = Instrument::Pad;
                        };
                    }
                );
                egui::ComboBox::from_id_salt("Effect")
                    .selected_text(format!("Effect"))
                    .show_ui(ui, |ui| {
                        if ui.button("Reverb").clicked(){

                        }
                        if ui.button("?").clicked(){

                        }
                        if ui.button("?").clicked(){

                        }
                        if ui.button("?").clicked(){

                        }
                    }
                );
                egui::ComboBox::from_id_salt("")
                    .selected_text(format!("??"))
                    .show_ui(ui, |ui| {
                        if ui.button("?").clicked(){

                        }
                    }
                );
                
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("sound test :");
                if ui.button("All sound").clicked(){
                    piano_test::run_all_notes();
                }
            });
            ui.separator();

        });
    }
}
