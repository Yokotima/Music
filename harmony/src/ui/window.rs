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

struct MyApp{
    name: String,
    instru: String,

}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "world".to_string(),
            instru: "None".to_string(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame:&mut eframe::Frame){
        egui::CentralPanel::default().show(ctx, |ui|{
            ui.heading("Test");
            ui.separator();
            
            egui::ComboBox::from_label("Select")
                .selected_text(format!("{}",self.instru))
                .show_ui(ui, |ui| {
                    if ui.button("piano").clicked(){
                        self.instru = "piano".to_string()
                    };
                    if ui.button("flute").clicked(){
                        self.instru = "flute".to_string()
                    };
                    if ui.button("basse").clicked(){
                        self.instru = "basse".to_string()
                    };
                    if ui.button("lead").clicked(){
                        self.instru = "lead".to_string()
                    };
                    if ui.button("pad").clicked(){
                        self.instru = "pad".to_string()
                    };
                }
            );

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("sound test :");
                if ui.button("All sound").clicked(){
                    piano_test::run_all_notes();
                }
            });
        });
    }
}
