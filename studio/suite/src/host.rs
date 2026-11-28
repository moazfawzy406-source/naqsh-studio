//! One window, several personas (spike): Deck and Layout hosted in one eframe window with a tab bar.
use deckcraft_engine::Session as DeckSession;
use deckcraft_ui_egui::{Services as DeckServices, SlideApp};
use vectorcraft_engine::Session as VecSession;
use vectorcraft_ui_egui::{Services as VecServices, VectorcraftApp};
use photocraft_ui_egui::PhotocraftApp;
use designcraft_engine::Session as DesignSession;
use designcraft_ui_egui::{DesignApp, Services as DesignServices};

#[derive(Clone, Copy, PartialEq)]
enum Persona {
    Layout,
    Deck,
    Vector,
    Photo,
}

struct Host {
    active: Persona,
    layout: DesignApp,
    deck: SlideApp,
    vector: VectorcraftApp,
    photo: Option<Box<PhotocraftApp>>,
    ar: bool,
    dark: bool,
    apply: bool,
    start: std::time::Instant,
    auto: bool,
}

impl Host {
    /// Photo is made the first time its tab opens: it installs its own look on the window's context then.
    fn photo_app(&mut self, ctx: &egui::Context) -> &mut Box<PhotocraftApp> {
        self.photo.get_or_insert_with(|| {
            PhotocraftApp::setup_context(ctx, Default::default());
            PhotocraftApp::with_context(ctx.clone(), photocraft_engine::Session::new(), Default::default())
        })
    }
    /// One language and one dark/light choice for every persona in the window.
    fn apply_prefs(&mut self) {
        deckcraft_ui_egui::ar_ui::set_arabic(self.ar);
        self.deck.ui.brightness = if self.dark { deckcraft_ui_egui::theme::Brightness::Dark } else { deckcraft_ui_egui::theme::Brightness::Light };
        self.layout.ui.language = if self.ar { "ar".to_owned() } else { String::new() };
        self.layout.ui.brightness = if self.dark { designcraft_ui_egui::theme::Brightness::Dark } else { designcraft_ui_egui::theme::Brightness::Light };
        self.vector.naqsh_set(if self.ar { "ar" } else { "en" }, self.dark);
    }
}

fn switches(ui: &mut egui::Ui, ar: &mut bool, dark: &mut bool) -> bool {
    let mut changed = false;
    ui.separator();
    if ui.button(if *ar { "EN" } else { "AR" }).clicked() {
        *ar = !*ar;
        changed = true;
    }
    if ui.button(if *dark { "Light" } else { "Dark" }).clicked() {
        *dark = !*dark;
        changed = true;
    }
    changed
}

impl eframe::App for Host {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self.active {
            Persona::Layout => self.layout.logic(ctx),
            Persona::Deck => self.deck.logic(ctx),
            Persona::Vector => vectorcraft_app::host_embed::logic(ctx, &mut self.vector),
            Persona::Photo => {
                let app = self.photo_app(ctx);
                eframe::App::logic(&mut **app, ctx, frame);
            }
        }
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        match self.active {
            Persona::Layout => self.layout.raw_input_hook(ctx, raw),
            Persona::Deck => self.deck.raw_input_hook(ctx, raw),
            Persona::Vector => {
                vectorcraft_app::host_embed::before_input(ctx, &mut self.vector);
                self.vector.raw_input(ctx, raw);
            }
            Persona::Photo => {
                if let Some(app) = self.photo.as_mut() {
                    eframe::App::raw_input_hook(&mut **app, ctx, raw);
                }
            }
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if self.apply {
            self.apply = false;
            self.apply_prefs();
        }
        match vectorcraft_ui_egui::SUITE_REQUEST.swap(0, std::sync::atomic::Ordering::Relaxed) {
            1 => self.active = Persona::Deck,
            2 => self.active = Persona::Layout,
            _ => {}
        }
        if self.auto {
            self.active = match (self.start.elapsed().as_secs() / 16) % 5 {
                0 => Persona::Layout,
                1 => Persona::Deck,
                2 => Persona::Vector,
                3 => Persona::Photo,
                _ => Persona::Layout,
            };
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
        }
        if !matches!(self.active, Persona::Vector | Persona::Photo) {
            egui::Panel::top("naqsh_personas").exact_size(30.0).show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    tab_row(ui, &mut self.active);
                    self.apply |= switches(ui, &mut self.ar, &mut self.dark);
                });
            });
        }
        match self.active {
            Persona::Layout => {
                self.layout.ui(ui);
                self.layout.show_file_dialogs(Some(&*frame));
            }
            Persona::Deck => self.deck.ui(ui),
            Persona::Vector => {
                vectorcraft_app::host_embed::before_ui(ui.ctx(), frame);
                self.vector.ui(ui);
                // Vector fills the whole window itself, so the tabs float over its top bar.
                let ctx = ui.ctx().clone();
                let mut next = self.active;
                egui::Area::new(egui::Id::new("naqsh_personas_float"))
                    .order(egui::Order::Foreground)
                    .anchor(egui::Align2::LEFT_TOP, egui::vec2(48.0, 4.0))
                    .show(&ctx, |ui| {
                        egui::Frame::popup(ui.style()).show(ui, |ui| ui.horizontal(|ui| {
                            tab_row(ui, &mut next);
                            self.apply |= switches(ui, &mut self.ar, &mut self.dark);
                        }));
                    });
                self.active = next;
                ctx.send_viewport_cmd(egui::ViewportCommand::Title("Naqsh Studio".to_owned()));
            }
            Persona::Photo => {
                let ctx = ui.ctx().clone();
                let app = self.photo_app(&ctx);
                eframe::App::ui(&mut **app, ui, frame);
                // Photo fills the whole window, so the tabs float at the top right.
                let mut next = self.active;
                egui::Area::new(egui::Id::new("naqsh_personas_float_photo"))
                    .order(egui::Order::Foreground)
                    .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 4.0))
                    .show(&ctx, |ui| {
                        egui::Frame::popup(ui.style()).show(ui, |ui| ui.horizontal(|ui| {
                            tab_row(ui, &mut next);
                            self.apply |= switches(ui, &mut self.ar, &mut self.dark);
                        }));
                    });
                self.active = next;
                ctx.send_viewport_cmd(egui::ViewportCommand::Title("Naqsh Studio".to_owned()));
            }
        }
    }
}

pub fn run() -> Result<(), String> {
    vectorcraft_ui_egui::SUITE_HOSTED.store(true, std::sync::atomic::Ordering::Relaxed);
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Naqsh Studio",
        options,
        Box::new(|cc| {
            if let Err(e) = vectorcraft_app::host_embed::install(&cc.egui_ctx) {
                eprintln!("naqsh: craft install failed: {e}");
            }
            let mut vector = VectorcraftApp::new(VecSession::new(), VecServices::default());
            vector.adopt_context(&cc.egui_ctx);
            Ok(Box::new(Host {
                vector,
                photo: None,
                ar: std::env::var("NAQSH_LANG").as_deref() == Ok("ar"),
                dark: std::env::var("NAQSH_THEME").as_deref() == Ok("dark"),
                apply: true,
                start: std::time::Instant::now(),
                auto: std::env::var_os("NAQSH_AUTOSWITCH").is_some(),
                active: if std::env::var("NAQSH_START").as_deref() == Ok("deck") { Persona::Deck } else { Persona::Layout },
                layout: DesignApp::new(DesignSession::new(), DesignServices::default()),
                deck: SlideApp::new(DeckSession::new(), DeckServices::default()),
            }))
        }),
    )
    .map_err(|e| e.to_string())
}

fn tab_row(ui: &mut egui::Ui, active: &mut Persona) {
    ui.selectable_value(active, Persona::Layout, "Layout");
    ui.selectable_value(active, Persona::Deck, "Deck");
    ui.selectable_value(active, Persona::Vector, "Vector");
    ui.selectable_value(active, Persona::Photo, "Photo");
}
