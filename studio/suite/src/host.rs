//! One window, several personas (spike): Deck and Layout hosted in one eframe window with a tab bar.
use deckcraft_engine::Session as DeckSession;
use deckcraft_ui_egui::{Services as DeckServices, SlideApp};
use vectorcraft_engine::Session as VecSession;
use vectorcraft_ui_egui::{Services as VecServices, VectorcraftApp};
use designcraft_engine::Session as DesignSession;
use designcraft_ui_egui::{DesignApp, Services as DesignServices};

#[derive(Clone, Copy, PartialEq)]
enum Persona {
    Layout,
    Deck,
    Vector,
}

struct Host {
    active: Persona,
    layout: DesignApp,
    deck: SlideApp,
    vector: VectorcraftApp,
    start: std::time::Instant,
    auto: bool,
}

impl eframe::App for Host {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.active {
            Persona::Layout => self.layout.logic(ctx),
            Persona::Deck => self.deck.logic(ctx),
            Persona::Vector => {}
        }
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        match self.active {
            Persona::Layout => self.layout.raw_input_hook(ctx, raw),
            Persona::Deck => self.deck.raw_input_hook(ctx, raw),
            Persona::Vector => self.vector.raw_input(ctx, raw),
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if self.auto {
            self.active = match (self.start.elapsed().as_secs() / 12) % 4 {
                0 => Persona::Layout,
                1 => Persona::Deck,
                2 => Persona::Vector,
                _ => Persona::Layout,
            };
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
        }
        egui::Panel::top("naqsh_personas").exact_size(30.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.selectable_value(&mut self.active, Persona::Layout, "Layout");
                ui.selectable_value(&mut self.active, Persona::Deck, "Deck");
                ui.selectable_value(&mut self.active, Persona::Vector, "Vector");
            });
        });
        match self.active {
            Persona::Layout => {
                self.layout.ui(ui);
                self.layout.show_file_dialogs(Some(&*frame));
            }
            Persona::Deck => self.deck.ui(ui),
            Persona::Vector => self.vector.ui(ui),
        }
    }
}

pub fn run() -> Result<(), String> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Naqsh Studio",
        options,
        Box::new(|cc| {
            let mut vector = VectorcraftApp::new(VecSession::new(), VecServices::default());
            vector.adopt_context(&cc.egui_ctx);
            Ok(Box::new(Host {
                vector,
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
