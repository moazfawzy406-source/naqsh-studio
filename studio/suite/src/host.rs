//! One window, several personas (spike): Deck and Layout hosted in one eframe window with a tab bar.
use deckcraft_engine::Session as DeckSession;
use deckcraft_ui_egui::{Services as DeckServices, SlideApp};
use designcraft_engine::Session as DesignSession;
use designcraft_ui_egui::{DesignApp, Services as DesignServices};

#[derive(Clone, Copy, PartialEq)]
enum Persona {
    Layout,
    Deck,
}

struct Host {
    active: Persona,
    layout: DesignApp,
    deck: SlideApp,
}

impl eframe::App for Host {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.active {
            Persona::Layout => self.layout.logic(ctx),
            Persona::Deck => self.deck.logic(ctx),
        }
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        match self.active {
            Persona::Layout => self.layout.raw_input_hook(ctx, raw),
            Persona::Deck => self.deck.raw_input_hook(ctx, raw),
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::Panel::top("naqsh_personas").exact_size(30.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.selectable_value(&mut self.active, Persona::Layout, "Layout");
                ui.selectable_value(&mut self.active, Persona::Deck, "Deck");
            });
        });
        match self.active {
            Persona::Layout => {
                self.layout.ui(ui);
                self.layout.show_file_dialogs(Some(&*frame));
            }
            Persona::Deck => self.deck.ui(ui),
        }
    }
}

pub fn run() -> Result<(), String> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Naqsh Studio",
        options,
        Box::new(|_cc| {
            Ok(Box::new(Host {
                active: if std::env::var("NAQSH_START").as_deref() == Ok("deck") { Persona::Deck } else { Persona::Layout },
                layout: DesignApp::new(DesignSession::new(), DesignServices::default()),
                deck: SlideApp::new(DeckSession::new(), DeckServices::default()),
            }))
        }),
    )
    .map_err(|e| e.to_string())
}
