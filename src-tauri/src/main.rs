//! Native app entry point for Tauri desktop/mobile targets.
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Launch the Tauri application runtime.
fn main() {
    german_kr_lib::run();
}
