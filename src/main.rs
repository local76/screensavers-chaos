#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod runner;
mod chaos;

fn main() {
    let effect = chaos::Chaos::new();
    runner::run_main(effect, "chaos");
}