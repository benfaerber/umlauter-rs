use ksni::{self, menu::StandardItem};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct UmlauterTray {
    enabled: Arc<AtomicBool>,
}

impl UmlauterTray {
    pub fn new(enabled: Arc<AtomicBool>) -> Self {
        Self { enabled }
    }
}

impl ksni::Tray for UmlauterTray {
    fn id(&self) -> String {
        "umlauter".into()
    }

    fn title(&self) -> String {
        if self.enabled.load(Ordering::Relaxed) {
            "Umlauter (active)".into()
        } else {
            "Umlauter (paused)".into()
        }
    }

    fn icon_name(&self) -> String {
        if self.enabled.load(Ordering::Relaxed) {
            "input-keyboard".into()
        } else {
            "input-keyboard-symbolic".into()
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let is_enabled = self.enabled.load(Ordering::Relaxed);

        vec![
            ksni::MenuItem::Standard(StandardItem {
                label: if is_enabled {
                    "Pause".into()
                } else {
                    "Resume".into()
                },
                activate: Box::new(|tray: &mut Self| {
                    let current = tray.enabled.load(Ordering::Relaxed);
                    tray.enabled.store(!current, Ordering::Relaxed);
                    eprintln!("umlauter {}", if !current { "resumed" } else { "paused" });
                }),
                ..Default::default()
            }),
            ksni::MenuItem::Standard(StandardItem {
                label: "Quit".into(),
                activate: Box::new(|_: &mut Self| {
                    std::process::exit(0);
                }),
                ..Default::default()
            }),
        ]
    }
}
