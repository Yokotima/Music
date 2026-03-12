use eframe::egui;
use eframe::epaint::Color32;
use crate::audio::piano_test;
use crate::audio::piano;
use crate::audio::piano::PianoNote;
use crate::audio::piano::play_sound;
use crate::audio::piano::stop_sound;
use std::time::Duration;
use crate::sequencer::sequencer::StepSequencer;

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
#[derive(PartialEq)]
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
    s: StepSequencer,
}


impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "world".to_string(),
            enum_instru: Instrument::Piano,
            s: StepSequencer::new(120.0, 16, 44_100),
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
                        if ui.button("Piano").clicked(){
                            self.enum_instru = Instrument::Piano;
                        };
                        if ui.button("Flute").clicked(){
                            self.enum_instru = Instrument::Flute;
                        };
                        if ui.button("Basse").clicked(){
                            self.enum_instru = Instrument::Bass;
                        };
                        if ui.button("Lead").clicked(){
                            self.enum_instru = Instrument::Lead;
                        };
                        if ui.button("Pad").clicked(){
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
            egui::Frame::none()
                .fill(egui::Color32::GRAY)
                .show(ui, |ui| {
                ui.scope(|ui| {
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::WHITE;
                    if ui.button("         ").clicked()
                    {
                        piano::play_sound(PianoNote::B4,0.75,&mut self.s );
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        piano::stop_sound(PianoNote::B4, &mut self.s);
                    }
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::BLACK;
                    if ui.button("       ").clicked()
                    {
                        piano::play_sound(PianoNote::As4,0.75,&mut self.s );
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        piano::stop_sound(PianoNote::As4, &mut self.s);
                    }
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::WHITE;
                    if ui.button("         ").clicked()
                    {
                        piano::play_sound(PianoNote::A4,0.75,&mut self.s );
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        piano::stop_sound(PianoNote::A4, &mut self.s);
                    }
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::BLACK;
                    if ui.button("       ").clicked()
                    {
                        piano::play_sound(PianoNote::Gs4,0.75,&mut self.s );
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        piano::stop_sound(PianoNote::Gs4, &mut self.s);
                    }
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::WHITE;
                    if ui.button("         ").clicked()
                    {
                        piano::play_sound(PianoNote::G4,0.75,&mut self.s );
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        piano::stop_sound(PianoNote::G4, &mut self.s);
                    }
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::BLACK;  
                    if ui.button("       ").clicked()
                    {
                        piano::play_sound(PianoNote::Fs4,0.75,&mut self.s );
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        piano::stop_sound(PianoNote::Fs4, &mut self.s);
                    }
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::WHITE;
                    if ui.button("         ").clicked()
                    {
                        piano::play_sound(PianoNote::F4,0.75,&mut self.s );
                        piano::stop_sound(PianoNote::F4, &mut self.s);
                    }
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::WHITE;
                    if ui.button("         ").clicked()
                    {
                        piano::play_sound(PianoNote::E4,0.75,&mut self.s );
                        piano::stop_sound(PianoNote::E4, &mut self.s);
                    }
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::BLACK;
                    if ui.button("       ").clicked()
                    {
                        piano::play_sound(PianoNote::Ds4,0.75,&mut self.s );
                        piano::stop_sound(PianoNote::Ds4, &mut self.s);
                    }
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::WHITE;
                    if ui.button("         ").clicked()
                    {
                        piano::play_sound(PianoNote::D4,0.75,&mut self.s );
                        piano::stop_sound(PianoNote::D4, &mut self.s);
                    }
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::BLACK;
                    if ui.button("       ").clicked()
                    {
                        play_note(PianoNote::Cs4 , 1.0);
                        piano::play_sound(PianoNote::Cs4,0.75,&mut self.s );
                        piano::stop_sound(PianoNote::Cs4, &mut self.s);
                    }
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::WHITE;
                    if ui.button("         ").clicked()
                    {
                        play_note(PianoNote::C4 , 1.0);
                    }
                });
            });
        });
    }
}


pub fn play_note(note: PianoNote, duration_secs: f32)
{
    let mut seq = StepSequencer::new(120.0,16,44_100);

    play_sound(note, 1.0, &mut seq);
    std::thread::sleep(Duration::from_secs_f32(duration_secs));
    stop_sound(note, &mut seq);
}

