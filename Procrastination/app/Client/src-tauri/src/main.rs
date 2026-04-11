// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod config;
use client_lib::run;

fn main() {
    run();
}