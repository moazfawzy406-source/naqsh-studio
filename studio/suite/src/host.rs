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
    vfont: Option<bool>,
    vtick: u32,
    toggled: u8,
    prev: Persona,
    st_step: usize,
    st_at: Option<std::time::Instant>,
    selftest: bool,
    photo_synced: Option<(bool, bool)>,
    toggle_env: bool,
}

impl Host {
    /// Photo is made the first time its tab opens: it installs its own look on the window's context then.
    fn photo_app(&mut self, ctx: &egui::Context) -> &mut Box<PhotocraftApp> {
        let (ar, dark) = (self.ar, self.dark);
        self.photo.get_or_insert_with(|| {
            // Photo reads these two when it starts.
            unsafe {
                std::env::set_var("NAQSH_LANG", if ar { "ar" } else { "en" });
                std::env::set_var("NAQSH_THEME", if dark { "dark" } else { "light" });
            }
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
            Persona::Vector => {
                vectorcraft_app::host_embed::logic(ctx, &mut self.vector);
                self.vector.tick(ctx);
            }
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
        // Every app paints with the one shared egui style; the app that is shown re-applies its own look.
        if self.active != self.prev {
            self.prev = self.active;
            // Fonts too: another tab may have replaced the shared font set.
            self.layout.styled = false;
            self.deck.styled = false;
        }
        if self.apply {
            self.apply = false;
            self.apply_prefs();
        }
        match vectorcraft_ui_egui::SUITE_REQUEST.swap(0, std::sync::atomic::Ordering::Relaxed) {
            1 => self.active = Persona::Deck,
            2 => self.active = Persona::Layout,
            _ => {}
        }
        if self.selftest {
            self.selftest_step(&ui.ctx().clone());
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(300));
        }
        if self.toggle_env {
            let s = self.start.elapsed().as_secs();
            if self.toggled == 0 && s >= 100 { self.toggled = 1; self.ar = !self.ar; self.apply = true; }
            if self.toggled == 1 && s >= 105 { self.toggled = 2; self.dark = !self.dark; self.apply = true; }
        }
        if self.auto {
            self.active = match (self.start.elapsed().as_secs() / 24) % 5 {
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
                if self.vfont != Some(self.ar) {
                    self.vfont = Some(self.ar);
                    vectorcraft_ui_egui::ui_fonts().ui_language(if self.ar { "ar" } else { "en" }).install(ui.ctx());
                }
                self.vector.ui(ui);
                self.vtick += 1;
                if self.vtick % 45 == 0 {
                    let want = if self.ar { "ar" } else { "en" };
                    use vectorcraft_ui_egui::i18n::LangExt as _;
                    let have = self.vector.ui_language().code().to_string();
                    eprintln!("naqsh: vector lang have={have} want={want} drawn={} rtl={} tr={:?}", vectorcraft_ui_egui::i18n::current().code(), vectorcraft_ui_egui::i18n::is_rtl(), vectorcraft_ui_egui::i18n::has(vectorcraft_ui_egui::i18n::current(), "Open"));
                    if have != want {
                        self.vector.naqsh_set(want, self.dark);
                    }
                }
                // Vector fills the whole window with its own panels, so the tabs float. Drag them anywhere.
                let ctx = ui.ctx().clone();
                let mut next = self.active;
                let pos = egui::pos2((ctx.content_rect().right() - 400.0).max(300.0), 3.0);
                egui::Area::new(egui::Id::new("naqsh_personas_float"))
                    .order(egui::Order::Foreground)
                    .movable(true)
                    .default_pos(pos)
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
                let (ar, dark) = (self.ar, self.dark);
                let synced = self.photo_synced;
                self.photo_app(&ctx);
                if synced.is_none() {
                    self.photo_synced = Some((ar, dark));
                } else if synced != Some((ar, dark)) {
                    unsafe {
                        std::env::set_var("NAQSH_LANG", if ar { "ar" } else { "en" });
                        std::env::set_var("NAQSH_THEME", if dark { "dark" } else { "light" });
                    }
                    if let Some(p) = self.photo.as_mut() {
                        let r = p.run("prefs.set", serde_json::json!({"values": {"interface.language": if ar { "ar" } else { "en" }, "interface.appearanceMode": if dark { "dark" } else { "light" }}}));
                        eprintln!("naqsh: photo prefs.set ar={ar} dark={dark} -> {r:?}");
                    }
                    self.photo_synced = Some((ar, dark));
                }
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
                vfont: None,
                vtick: 0,
                toggled: 0,
                prev: Persona::Layout,
                st_step: 0,
                st_at: None,
                selftest: std::env::var_os("NAQSH_SELFTEST").is_some(),
                photo_synced: None,
                toggle_env: std::env::var_os("NAQSH_TOGGLE").is_some(),
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

fn st_log(what: &str, r: Result<serde_json::Value, String>) {
    let s = match r {
        Ok(v) => format!("OK {}", v.to_string().chars().take(300).collect::<String>()),
        Err(e) => format!("ERR {}", e.chars().take(300).collect::<String>()),
    };
    eprintln!("selftest: {what} -> {s}");
}

impl Host {
    /// CI functional test (NAQSH_SELFTEST): make a disposable document in each app, switch tabs away and back,
    /// save it to a temp folder, close it, open the saved file again. Every result goes to the log.
    fn selftest_step(&mut self, ctx: &egui::Context) {
        use serde_json::json;
        let wait = |me: &mut Host, secs: u64| me.st_at = Some(std::time::Instant::now() + std::time::Duration::from_secs(secs));
        if let Some(at) = self.st_at {
            if std::time::Instant::now() < at {
                return;
            }
        }
        let dir = std::env::temp_dir().join("naqsh-selftest");
        let path = |n: &str| dir.join(n).to_string_lossy().to_string();
        let step = self.st_step;
        self.st_step += 1;
        eprintln!("selftest: step {step}");
        match step {
            0 => {
                let _ = std::fs::create_dir_all(&dir);
                self.active = Persona::Vector;
                wait(self, 6);
            }
            1 => {
                st_log("vector file.new", self.vector.run("file.new", json!({"width": 600, "height": 400})));
                st_log("vector rectangle", self.vector.run("shape.rectangle", json!({"x": 100, "y": 100, "width": 200, "height": 120})));
                st_log("vector select.all", self.vector.run("select.all", json!({})));
                st_log("vector file.list", self.vector.run("file.list", json!({})));
                wait(self, 3);
            }
            2 => {
                self.active = Persona::Deck;
                wait(self, 5);
            }
            3 => {
                st_log("deck file.new", self.deck.run("file.new", json!({})));
                st_log("deck slide.new", self.deck.run("slide.new", json!({})));
                wait(self, 3);
            }
            4 => {
                self.active = Persona::Layout;
                wait(self, 5);
            }
            5 => {
                st_log("layout file.new", self.layout.run("file.new", json!({})));
                st_log("layout frame.create", self.layout.run("frame.create", json!({"rect": [36, 36, 200, 200], "content": "text", "text": "naqsh test"})));
                wait(self, 3);
            }
            6 => {
                self.active = Persona::Photo;
                wait(self, 6);
            }
            7 => {
                let p = self.photo_app(ctx);
                st_log("photo file.new", p.run("file.new", json!({"width": 200, "height": 150})));
                st_log("photo paint.stroke", p.run("paint.stroke", json!({"points": [[20, 20], [100, 80]], "size": 9, "hardness": 1.0})));
                wait(self, 3);
            }
            8 => {
                // Back to Vector: is the document still there?
                self.active = Persona::Vector;
                wait(self, 5);
            }
            9 => {
                st_log("vector (after tabs) file.list", self.vector.run("file.list", json!({})));
                st_log("vector (after tabs) select.all", self.vector.run("select.all", json!({})));
                st_log("vector file.saveAs", self.vector.run("file.saveAs", json!({"path": path("v.svg"), "format": "svg"})));
                self.active = Persona::Deck;
                wait(self, 4);
            }
            10 => {
                st_log("deck (after tabs) slide.new again", self.deck.run("slide.new", json!({})));
                st_log("deck file.saveAs", self.deck.run("file.saveAs", json!({"path": path("d.pptx")})));
                self.active = Persona::Layout;
                wait(self, 4);
            }
            11 => {
                st_log("layout file.saveAs", self.layout.run("file.saveAs", json!({"path": path("l.idml")})));
                self.active = Persona::Photo;
                wait(self, 4);
            }
            12 => {
                let p = self.photo_app(ctx);
                st_log("photo file.saveACopy", p.run("file.saveACopy", json!({"path": path("p.psd")})));
                wait(self, 2);
            }
            13 => {
                for n in ["v.svg", "d.pptx", "l.idml", "p.psd"] {
                    let size = std::fs::metadata(dir.join(n)).map(|m| m.len()).ok();
                    eprintln!("selftest: file {n} size {size:?}");
                }
                st_log("vector file.close", self.vector.run("file.close", json!({})));
                st_log("vector file.open", self.vector.run("file.open", json!({"path": path("v.svg")})));
                st_log("vector select.all (reopened)", self.vector.run("select.all", json!({})));
                st_log("vector file.list (reopened)", self.vector.run("file.list", json!({})));
                self.active = Persona::Vector;
                wait(self, 5);
            }
            14 => {
                st_log("deck file.close", self.deck.run("file.close", json!({})));
                st_log("deck file.open", self.deck.run("file.open", json!({"path": path("d.pptx")})));
                self.active = Persona::Deck;
                wait(self, 5);
            }
            15 => {
                st_log("layout file.close", self.layout.run("file.close", json!({})));
                st_log("layout file.open", self.layout.run("file.open", json!({"path": path("l.idml")})));
                self.active = Persona::Layout;
                wait(self, 5);
            }
            16 => {
                let p = self.photo_app(ctx);
                st_log("photo file.close", p.run("file.close", json!({})));
                st_log("photo file.open", p.run("file.open", json!({"path": path("p.psd")})));
                self.active = Persona::Photo;
                wait(self, 6);
            }
            _ => {
                eprintln!("selftest: DONE");
                self.selftest = false;
            }
        }
    }
}
