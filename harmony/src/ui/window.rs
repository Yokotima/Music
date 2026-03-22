use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, Rounding, Sense, Stroke, StrokeKind, vec2};

use crate::audio::piano_test;
use crate::audio::note::{MidiNote, ALL_NOTES};
use crate::audio::play::play;
use crate::audio::instruments::InstrumentKind;

const BG: Color32 = Color32::from_rgb(18,18,20);
const TOOLBAR_BG: Color32 = Color32::from_rgb(26,26,30);
const PIANO_BG: Color32 = Color32::from_rgb(22,22,25);
const GRID_BG: Color32 = Color32::from_rgb(28,28,32);
const DIVIDER: Color32 = Color32::from_rgb(55,55,62);

const WHITE_KEY: Color32 = Color32::from_rgb(238,235,230);
const BLACK_KEY: Color32 = Color32::from_rgb(28,28,32);

const PIANO_W: f32 = 72.0;
const NAV_H: f32 = 28.0;
const BLACK_W_FRAC: f32 = 0.58;

const NOTES_PER_PAGE: usize = 22;

fn is_black(midi: u8) -> bool {
    matches!(midi % 12, 1 | 3 | 6 | 8 | 10)
}

struct MyApp {
    enum_instru: InstrumentKind,
    piano_page: usize,

    part_piano: [[bool;255];88],
    part_flute: [[bool;255];88],
    part_bass: [[bool;255];88],
    part_pad: [[bool;255];88],
    part_lead: [[bool;255];88],
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            enum_instru: InstrumentKind::Piano,
            piano_page: 0,

            part_piano: [[false;255];88],
            part_flute: [[false;255];88],
            part_bass: [[false;255];88],
            part_pad: [[false;255];88],
            part_lead: [[false;255];88],
        }
    }
}

pub fn window() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0,600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Harmony",
        options,
        Box::new(|_| Ok(Box::new(MyApp::default())))
    )
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx:&egui::Context, _frame:&mut eframe::Frame) {

        ctx.set_visuals(egui::Visuals::dark());

        egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(BG))
        .show(ctx, |ui| {

            let total = ui.max_rect();

            let toolbar_rect = Rect::from_min_size(
                total.min,
                vec2(total.width(),44.0)
            );

            self.draw_toolbar(ui,toolbar_rect);

            let body_top = total.min.y + 44.0;

            let body_rect = Rect::from_min_max(
                Pos2::new(total.min.x,body_top),
                total.max
            );

            let piano_rect = Rect::from_min_size(
                body_rect.min,
                vec2(PIANO_W,body_rect.height())
            );

            ui.painter().line_segment(
                [
                    Pos2::new(body_rect.min.x+PIANO_W,body_rect.min.y),
                    Pos2::new(body_rect.min.x+PIANO_W,body_rect.max.y)
                ],
                Stroke::new(2.0,DIVIDER)
            );

            let grid_rect = Rect::from_min_max(
                Pos2::new(body_rect.min.x+PIANO_W+2.0,body_rect.min.y),
                body_rect.max
            );

            self.draw_piano(ui,piano_rect);
            self.draw_grid(ui,grid_rect);

        });
    }
}

impl MyApp {
    fn draw_toolbar(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.allocate_ui_at_rect(rect, |ui| {
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
                            self.enum_instru = InstrumentKind::Piano;
                        }
                        if ui.button("Flute").clicked() {
                            self.enum_instru = InstrumentKind::Flute;
                        }
                        if ui.button("Bass").clicked() {
                            self.enum_instru = InstrumentKind::Bass;
                        }
                        if ui.button("Lead").clicked() {
                           self.enum_instru = InstrumentKind::Lead;
                        }
                        if ui.button("Pad").clicked() {
                            self.enum_instru = InstrumentKind::Pad;
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
                if ui.button("Play").clicked()
                {
                    //play sound
                }
                if ui.button("Stop").clicked()
                {
                    //stop sound
                }
                ui.separator();
                ui.label("sound test :");
                if ui.button("All sound").clicked() {
                    piano_test::run_all_notes();
                }
            });
        });
    }

    fn draw_piano(&mut self, ui:&mut egui::Ui, rect:Rect) {
        let page_start = self.piano_page * NOTES_PER_PAGE;
        let page_end = (page_start + NOTES_PER_PAGE).min(88);

        let visible = &ALL_NOTES[page_start..page_end];
	let n_visible  = page_end - page_start;

        let nav_rect = Rect::from_min_size(rect.min,vec2(PIANO_W,NAV_H));
	let keys_top  = rect.min.y + NAV_H;
    	let keys_rect = Rect::from_min_max(Pos2::new(rect.min.x, keys_top), rect.max);

    	let row_h   = keys_rect.height() / n_visible as f32;
    	let black_w = PIANO_W * BLACK_W_FRAC;
    	let btn_w   = PIANO_W / 4.0;

        let mut clicked_page = None;

        for i in 0..4 {
            let btn_rect = Rect::from_min_size(
                Pos2::new(nav_rect.min.x + i as f32 * btn_w, nav_rect.min.y),
                vec2(btn_w, NAV_H),
            );

            if ui.allocate_rect(btn_rect, Sense::click()).clicked() {
                clicked_page = Some(i);
            }
        }

        if let Some(i) = clicked_page {
            self.piano_page = i;
        }

        let p = ui.painter();

        p.rect_filled(rect, Rounding::ZERO, PIANO_BG);
        p.rect_filled(nav_rect, Rounding::ZERO, Color32::from_gray(80));
        p.line_segment(
            [nav_rect.left_bottom(), 
            nav_rect.right_bottom()],
            Stroke::new(1.0, DIVIDER),
        );

        for i in 0..4 {
            let btn_rect = Rect::from_min_size(
                Pos2::new(nav_rect.min.x + i as f32 * btn_w, 
                nav_rect.min.y),
                vec2(btn_w, NAV_H),
            );

            let active = self.piano_page == i;
            let bg_color = if active { Color32::from_gray(120) } else { Color32::from_gray(60) };
            p.rect_filled(btn_rect, Rounding::same(3), bg_color);

            if i > 0 {
                p.line_segment(
                    [btn_rect.left_top(),
                    btn_rect.left_bottom()],
                    Stroke::new(1.0, DIVIDER),
                );
            }
            p.text(
                btn_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{}", i + 1),
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );
        }

        // First, the White key
        for (row,&note) in visible.iter().enumerate() {
            let midi = note.midi();
            if is_black(midi) { continue; }
            let y = keys_rect.min.y + row as f32 * row_h;
            let key_rect = Rect::from_min_size(
                Pos2::new(keys_rect.min.x,y),
                vec2(PIANO_W,row_h)
            );

            let resp = ui.allocate_rect(key_rect,Sense::click());
            // Click => Sound
            if resp.clicked() {
                let inst = self.enum_instru;
                std::thread::spawn(move || { play(note,inst,0.5); });
            }
            // Paint in white the box
            ui.painter().rect_filled(key_rect,Rounding::ZERO,WHITE_KEY);
            // Paint the border in black
            ui.painter().rect_stroke(
                key_rect,
                Rounding::ZERO,
                Stroke::new(1.0,Color32::BLACK),
                StrokeKind::Outside,
            );
            // Add the kawai text
            ui.painter().text(
                Pos2::new(key_rect.right()-4.0,key_rect.center().y),
                egui::Align2::RIGHT_CENTER,
                note.name(),
                egui::FontId::proportional((row_h*0.45).min(9.0)),
                Color32::BLACK
            );
        }

        // Now the black note EXACTLY the same as white!
        for (row,&note) in visible.iter().enumerate() {
            let midi = note.midi();
            if !is_black(midi) { continue; }
            let y = keys_rect.min.y + row as f32 * row_h;
            let key_rect = Rect::from_min_size(
                Pos2::new(keys_rect.min.x,y),
                vec2(black_w,row_h)
            );

            let resp = ui.allocate_rect(key_rect,Sense::click());
            if resp.clicked() {
                let inst = self.enum_instru;
                std::thread::spawn(move || { play(note,inst,0.5); });
            }

            ui.painter().rect_filled(
                key_rect,
                Rounding {nw:0,sw:0,ne:3,se:3},
                BLACK_KEY
            );

            ui.painter().rect_stroke(
                key_rect,
                Rounding {nw:0,sw:0,ne:3,se:3},
                Stroke::new(1.0,Color32::from_gray(80)),
                StrokeKind::Outside,
            );

            ui.painter().text(
                Pos2::new(key_rect.right()-3.0,key_rect.center().y),
                egui::Align2::RIGHT_CENTER,
                note.name(),
                egui::FontId::proportional((row_h*0.42).min(8.5)),
                Color32::WHITE
            );
        }
    }

    fn draw_grid(&mut self, ui:&mut egui::Ui, rect:Rect) {
        let page_start = self.piano_page * NOTES_PER_PAGE;
        let page_end = (page_start + NOTES_PER_PAGE).min(88);

        ui.painter().rect_filled(rect,Rounding::ZERO,GRID_BG);
        ui.allocate_ui_at_rect(rect, |ui| {
            let part = match self.enum_instru {
                InstrumentKind::Piano => &mut self.part_piano,
                InstrumentKind::Flute => &mut self.part_flute,
                InstrumentKind::Bass => &mut self.part_bass,
                InstrumentKind::Pad => &mut self.part_pad,
                InstrumentKind::Lead => &mut self.part_lead,
            };

            egui::Grid::new("partition_grid")
                .num_columns(255)
                .show(ui, |ui| {
                    for note_idx in page_start..page_end {
                        for step in part[note_idx].iter_mut() {
                            ui.toggle_value(step, "  ");
                        }
                ui.end_row();
                }
            });
        });
    }
}

pub fn play_note(note:MidiNote,duration_secs:f32){
    play(note,InstrumentKind::Piano,duration_secs);
}
