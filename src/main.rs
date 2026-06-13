#![allow(dead_code)]
#![allow(unused_imports)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod runner;
mod chaos;

#[cfg(test)]
mod tests_perf;

fn main() {
    let effect = chaos::Chaos::new();
    crate::runner::screensaver_runner::run_main(effect, "chaos");
}
