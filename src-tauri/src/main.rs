// Prevents additional console window on Windows in release mode.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    fpga_flow_mind_lib::run();
}
