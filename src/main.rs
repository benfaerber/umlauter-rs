mod config;
mod keyboard;
mod tray;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn main() {
    let config = config::Config::load();
    eprintln!("loaded {} accent mappings", config.mappings.len());

    let enabled = Arc::new(AtomicBool::new(true));

    let listener =
        keyboard::KeyboardListener::new(config.mappings, config.max_trigger_len, enabled.clone());
    listener.run();

    eprintln!("umlauter is running");

    let tray_enabled = enabled.clone();
    std::thread::spawn(move || {
        let service = ksni::TrayService::new(tray::UmlauterTray::new(tray_enabled));
        match service.run() {
            Ok(()) => {}
            Err(e) => eprintln!("tray failed (running without tray): {e}"),
        }
    });

    // Keep main thread alive
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}
