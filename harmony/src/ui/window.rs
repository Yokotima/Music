use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, CornerRadius, Sense, Stroke, StrokeKind, vec2};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};

use crate::audio::note::{ALL_NOTES};
use crate::audio::play::play;
use crate::audio::instruments::InstrumentKind;
use crate::sequencer::sequencer::StepSequencer;
use crate::files::save_to_json::{
    save_to_json, load_from_json, export_to_wav,
    Project, Track as SaveTrack, Step as SaveStep,
};
use crate::audio::effects::EffectMode;

const BG: Color32 = Color32::from_rgb(18,18,20);
const TOOLBAR_BG: Color32 = Color32::from_rgb(26,26,30);
const PIANO_BG: Color32 = Color32::from_rgb(22,22,25);
const GRID_BG: Color32 = Color32::from_rgb(28,28,32);
const DIVIDER: Color32 = Color32::from_rgb(55,55,62);

const WHITE_KEY: Color32 = Color32::from_rgb(238,235,230);
const BLACK_KEY: Color32 = Color32::from_rgb(28,28,32);

const STEP_ON: Color32 = Color32::from_rgb(80, 160, 255);
const STEP_OFF: Color32 = Color32::from_rgb(45, 45, 52);

const STEP_COUNT: usize = 64;
const SAMPLE_RATE: u32 = 44_100;

type Grid = [[Option<u8>; STEP_COUNT]; 88];

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

    part_piano: Grid,
    part_flute: Grid,
    part_bass: Grid,
    part_pad: Grid,
    part_lead: Grid,

    sequencer: Arc<Mutex<StepSequencer>>,
    _stream: Option<cpal::Stream>,
    is_playing: bool,

    enum_effect: EffectMode,
    effect_wet: f32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            enum_instru: InstrumentKind::Piano,
            piano_page: 0,

            part_piano: [[None; STEP_COUNT]; 88],
            part_flute: [[None; STEP_COUNT]; 88],
            part_bass: [[None; STEP_COUNT]; 88],
            part_pad: [[None; STEP_COUNT]; 88],
            part_lead: [[None; STEP_COUNT]; 88],

            sequencer: Arc::new(Mutex::new(StepSequencer::new(60.0, STEP_COUNT, SAMPLE_RATE))),
            _stream: None,
            is_playing: false,

            enum_effect: EffectMode::None,
            effect_wet: 0.5,
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
        .frame(egui::Frame::NONE.fill(BG))
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
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.painter().rect_filled(rect, CornerRadius::ZERO, TOOLBAR_BG);
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("Files")
                    .selected_text("Files")
                    .show_ui(ui, |ui| {
                        if ui.button("Save").clicked() {
                            let project = self.build_project();
                            if let Err(e) = save_to_json(&project, "project.json") {
                                eprintln!("[save] {e}");
                            }
                        }
                        if ui.button("Import").clicked() {
                            self.stop_playback();
                            match load_from_json("project.json") {
                                Ok(project) => self.load_project(project),
                                Err(e) => eprintln!("[load] {e}"),
                            }
                        }
                        if ui.button("Export").clicked() {
                            let project = self.build_project();
                            if let Err(e) = export_to_wav(&project, "project.wav") {
                                eprintln!("[export] {e}");
                            }
                        }
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

                let effect_label = match self.enum_effect {
                    EffectMode::None => "Effect: None",
                    EffectMode::Reverb => "Effect: Reverb",
                    EffectMode::Delay => "Effect: Delay",
                    EffectMode::Chorus => "Effect: Chorus",
                };
                egui::ComboBox::from_id_salt("Effect")
                    .selected_text(effect_label)
                    .show_ui(ui, |ui| {
                        for (label, mode) in [
                            ("None", EffectMode::None),
                            ("Reverb", EffectMode::Reverb),
                            ("Delay", EffectMode::Delay),
                            ("Chorus", EffectMode::Chorus),
                        ] {
                            if ui.selectable_label(self.enum_effect == mode, label).clicked() {
                                self.enum_effect = mode;
                                self.apply_effect_to_sequencer();
                            }
                        }
                    });
                if self.enum_effect != EffectMode::None {
                    ui.label("Wet:");
                    if ui.add(
                        egui::Slider::new(&mut self.effect_wet, 0.0..=1.0)
                            .show_value(false)
                            .fixed_decimals(2)
                        ).changed() {
                        self.apply_effect_to_sequencer();
                    }
                }

                if ui.button("Play").clicked()
                {
                    //play sound
                    self.start_playback();
                }
                if ui.button("Stop").clicked()
                {
                    //stop sound
                    self.stop_playback();
                }

                ui.separator();
                ui.label("sound test :");
                if ui.button("All sound").clicked() {
                    let inst = self.enum_instru;
                    let eff = self.enum_effect;
                    std::thread::spawn(move || {
                        for &note in &ALL_NOTES {
                            play(note, inst, 0.4, eff);
                        }
                    });
                }
                if ui.button("Clear track").clicked() {
                    self.stop_playback();
                    let part: &mut Grid = match self.enum_instru {
                        InstrumentKind::Piano => &mut self.part_piano,
                        InstrumentKind::Flute => &mut self.part_flute,
                        InstrumentKind::Bass => &mut self.part_bass,
                        InstrumentKind::Pad => &mut self.part_pad,
                        InstrumentKind::Lead => &mut self.part_lead,
                    };
                    for row in part.iter_mut() {
                        for step in row.iter_mut() {
                            *step = None;
                        }
                    }
                }
            });
        });
    }

    fn apply_effect_to_sequencer(&mut self) {
        if let Ok(mut seq) = self.sequencer.try_lock() {
            for track in seq.tracks.iter_mut() {
                track.engine.fx.mode = self.enum_effect;
                track.engine.fx.wet_mix = self.effect_wet;
            }
        }
    }

    fn build_project(&self) -> Project
    {
        let instruments: [(InstrumentKind, &Grid); 5] = [
            (InstrumentKind::Piano, &self.part_piano),
            (InstrumentKind::Flute, &self.part_flute),
            (InstrumentKind::Bass, &self.part_bass),
            (InstrumentKind::Pad, &self.part_pad),
            (InstrumentKind::Lead, &self.part_lead),
        ];

        let mut tracks = Vec::new();

        for (kind, grid) in &instruments {
            for note_idx in 0..88 {
                let row = &grid[note_idx];
                if !row.iter().any(|s| s.is_some()) { continue; }

                let midi = ALL_NOTES[note_idx].midi();
                let steps = (0..STEP_COUNT).map(|step_idx| {
                    let active = row[step_idx].is_some();
                    SaveStep {
                        active,
                        note: if active { Some(midi) } else { None },
                        velocity: None,
                    }
                }).collect();

                tracks.push(SaveTrack {
                    engine: *kind,
                    steps,
                    default_note: midi,
                    default_velocity: 0.8,
                    muted: false,
                });
            }
        }

        Project {
            name: "Harmony Project".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            tracks,
        }
    }

    fn load_project(&mut self, project: Project)
    {
        self.part_piano = [[None; STEP_COUNT]; 88];
        self.part_flute = [[None; STEP_COUNT]; 88];
        self.part_bass = [[None; STEP_COUNT]; 88];
        self.part_pad = [[None; STEP_COUNT]; 88];
        self.part_lead = [[None; STEP_COUNT]; 88];

        for track in project.tracks {
            let grid: &mut Grid = match track.engine {
                InstrumentKind::Piano => &mut self.part_piano,
                InstrumentKind::Flute => &mut self.part_flute,
                InstrumentKind::Bass => &mut self.part_bass,
                InstrumentKind::Pad => &mut self.part_pad,
                InstrumentKind::Lead => &mut self.part_lead,
            };

            let note_idx_opt = (0..88).find(|&i| {
                ALL_NOTES[i].midi() == track.default_note
            });
            let note_idx = match note_idx_opt {
                Some(i) => i,
                None => continue,
            };

            for (step_idx, step) in track.steps.iter().enumerate() {
                if step_idx >= STEP_COUNT { break; }
                if step.active {
                    grid[note_idx][step_idx] = Some(track.default_note);
                }
            }
        }
    }

    fn start_playback(&mut self)
    {
        {
            let mut seq = self.sequencer.lock().unwrap();
            seq.stop();
            seq.tracks.clear();

            let instruments: [InstrumentKind; 5] = [
                InstrumentKind::Piano,
                InstrumentKind::Flute,
                InstrumentKind::Bass,
                InstrumentKind::Pad,
                InstrumentKind::Lead,
            ];
            let grids: [&Grid; 5] = [
                &self.part_piano,
                &self.part_flute,
                &self.part_bass,
                &self.part_pad,
                &self.part_lead,
            ];

            for (kind, grid) in instruments.iter().zip(grids.iter()) {
                for note_idx in 0..88 {
                    let row = &grid[note_idx];
                    if !row.iter().any(|s| s.is_some()) { continue; }

                    let midi = ALL_NOTES[note_idx].midi();
                    let track_idx = seq.add_track(*kind, midi);

                    for step_idx in 0..STEP_COUNT {
                        if row[step_idx].is_some() {
                            seq.set_step_note(track_idx, step_idx, midi);
                        }
                    }

                    // Apply current effect to this track
                    seq.tracks[track_idx].engine.fx.mode = self.enum_effect;
                    seq.tracks[track_idx].engine.fx.wet_mix = self.effect_wet;
                }
            }

            seq.looping = false;
            seq.play();
        }

        let seq_clone = Arc::clone(&self.sequencer);

        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => { eprintln!("[play] No output device"); return; }
        };

        let config = StreamConfig {
            channels: 2,
            sample_rate: SampleRate(SAMPLE_RATE),
            buffer_size: BufferSize::Fixed(1024),
        };

        let stream = device.build_output_stream(
            &config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut seq = seq_clone.lock().unwrap();
                for frame in output.chunks_mut(2) {
                    let (l, r) = seq.next_sample();
                    if frame.len() >= 2 {
                        frame[0] = l;
                        frame[1] = r;
                    }
                }
            },
            |err| eprintln!("[play] Stream error: {err}"),
            None,
        ).unwrap();

        stream.play().unwrap();
        self._stream = Some(stream);
        self.is_playing = true;
    }

    fn stop_playback(&mut self)
    {
        {
            let mut seq = self.sequencer.lock().unwrap();
            seq.stop();
        }
        self._stream = None;
        self.is_playing = false;
    }

    fn draw_piano(&mut self, ui:&mut egui::Ui, rect:Rect) {
        let page_start = self.piano_page * NOTES_PER_PAGE;
        let page_end = (page_start + NOTES_PER_PAGE).min(88);

        let visible = &ALL_NOTES[page_start..page_end];
        let n_visible = page_end - page_start;

        let nav_rect = Rect::from_min_size(rect.min,vec2(PIANO_W,NAV_H));
        let keys_top = rect.min.y + NAV_H;
        let keys_rect = Rect::from_min_max(Pos2::new(rect.min.x, keys_top), rect.max);

        let row_h = keys_rect.height() / n_visible as f32;
        let black_w = PIANO_W * BLACK_W_FRAC;
        let btn_w = PIANO_W / 4.0;

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

        p.rect_filled(rect, CornerRadius::ZERO, PIANO_BG);
        p.rect_filled(nav_rect, CornerRadius::ZERO, Color32::from_gray(80));
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
            p.rect_filled(btn_rect, CornerRadius::same(3), bg_color);

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
                let eff = self.enum_effect;
                std::thread::spawn(move || { play(note,inst,0.5,eff); });
            }
            // Paint in white the box
            ui.painter().rect_filled(key_rect,CornerRadius::ZERO,WHITE_KEY);
            // Paint the border in black
            ui.painter().rect_stroke(
                key_rect,
                CornerRadius::ZERO,
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
                let eff = self.enum_effect;
                std::thread::spawn(move || { play(note,inst,0.5,eff); });
            }

            ui.painter().rect_filled(
                key_rect,
                CornerRadius {nw:0,sw:0,ne:3,se:3},
                BLACK_KEY
            );

            ui.painter().rect_stroke(
                key_rect,
                CornerRadius {nw:0,sw:0,ne:3,se:3},
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
        let n_visible = page_end - page_start;

        let row_h = rect.height() / n_visible as f32;
        let col_w = rect.width() / STEP_COUNT as f32;

        ui.painter().rect_filled(rect,CornerRadius::ZERO,GRID_BG);

        let part: &mut Grid = match self.enum_instru {
            InstrumentKind::Piano => &mut self.part_piano,
            InstrumentKind::Flute => &mut self.part_flute,
            InstrumentKind::Bass => &mut self.part_bass,
            InstrumentKind::Pad => &mut self.part_pad,
            InstrumentKind::Lead => &mut self.part_lead,
        };

        for row in 0..n_visible {
            let note_idx = page_start + row;
            let midi = ALL_NOTES[note_idx].midi();
            let y = rect.min.y + row as f32 * row_h;

            let row_bg = if is_black(midi) {
                Color32::from_rgb(24, 24, 28)
            } else {
                Color32::from_rgb(32, 32, 37)
            };
            ui.painter().rect_filled(
                Rect::from_min_size(Pos2::new(rect.min.x, y), vec2(rect.width(), row_h)),
                CornerRadius::ZERO,
                row_bg,
            );

            for step_idx in 0..STEP_COUNT {
                let x = rect.min.x + step_idx as f32 * col_w;
                let cell_rect = Rect::from_min_size(
                    Pos2::new(x + 1.0, y + 1.0),
                    vec2(col_w - 2.0, row_h - 2.0),
                );

                let is_active = part[note_idx][step_idx].is_some();
                ui.painter().rect_filled(cell_rect, CornerRadius::same(2),
                    if is_active { STEP_ON } else { STEP_OFF });

                if is_active && col_w > 20.0 {
                    ui.painter().text(
                        cell_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        ALL_NOTES[note_idx].name(),
                        egui::FontId::proportional((row_h * 0.4).min(8.0)),
                        Color32::WHITE,
                    );
                }

                let resp = ui.allocate_rect(cell_rect, Sense::click());
                if resp.clicked() {
                    part[note_idx][step_idx] = if is_active {
                        None
                    } else {
                        Some(midi)
                    };
                }
            }
        }
    }
}
