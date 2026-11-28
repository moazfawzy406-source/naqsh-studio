//! Naqsh Studio: one program, four apps. `NAQSH_APP` picks the app (vector, deck, layout, photo); default vector.
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]
use std::process::ExitCode;
mod host;

fn report<E: std::fmt::Display>(r: Result<(), E>) -> ExitCode {
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    match std::env::var("NAQSH_APP").unwrap_or_default().as_str() {
        "studio" => report(host::run()),
        "deck" => report(deckcraft_app::main()),
        "layout" => designcraft_app::main(),
        "photo" => report(photocraft_app::main()),
        _ => vectorcraft_app::main(),
    }
}
