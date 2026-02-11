use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, Device, EventType, InputEvent, Key};
use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

struct KeyEvent {
    key: Key,
    value: i32,
}

pub struct KeyboardListener {
    mappings: HashMap<String, String>,
    max_trigger_len: usize,
    enabled: Arc<AtomicBool>,
}

fn keycode_to_char(key: Key, shift: bool) -> Option<char> {
    let c = match key {
        Key::KEY_A => 'a',
        Key::KEY_B => 'b',
        Key::KEY_C => 'c',
        Key::KEY_D => 'd',
        Key::KEY_E => 'e',
        Key::KEY_F => 'f',
        Key::KEY_G => 'g',
        Key::KEY_H => 'h',
        Key::KEY_I => 'i',
        Key::KEY_J => 'j',
        Key::KEY_K => 'k',
        Key::KEY_L => 'l',
        Key::KEY_M => 'm',
        Key::KEY_N => 'n',
        Key::KEY_O => 'o',
        Key::KEY_P => 'p',
        Key::KEY_Q => 'q',
        Key::KEY_R => 'r',
        Key::KEY_S => 's',
        Key::KEY_T => 't',
        Key::KEY_U => 'u',
        Key::KEY_V => 'v',
        Key::KEY_W => 'w',
        Key::KEY_X => 'x',
        Key::KEY_Y => 'y',
        Key::KEY_Z => 'z',
        _ => return None,
    };
    if shift {
        Some(c.to_ascii_uppercase())
    } else {
        Some(c)
    }
}

fn is_shift(key: Key) -> bool {
    matches!(key, Key::KEY_LEFTSHIFT | Key::KEY_RIGHTSHIFT)
}

fn find_keyboard_devices() -> Vec<(String, Device)> {
    let mut devices = Vec::new();
    let input_dir = "/dev/input";

    let entries = match fs::read_dir(input_dir) {
        Ok(e) => e,
        Err(_) => return devices,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.starts_with("event") {
            continue;
        }

        if let Ok(device) = Device::open(&path) {
            let dev_name = device.name().unwrap_or("unknown").to_string();

            if dev_name.contains("umlauter") {
                eprintln!("  skipping our own virtual device: {}", dev_name);
                continue;
            }

            if let Some(keys) = device.supported_keys() {
                if keys.contains(Key::KEY_A) && keys.contains(Key::KEY_Z) {
                    eprintln!("  found keyboard: {} ({})", dev_name, path.display());
                    devices.push((dev_name, device));
                }
            }
        }
    }

    devices
}

fn create_virtual_keyboard() -> VirtualDevice {
    let mut keys = AttributeSet::<Key>::new();
    for code in 0..256 {
        keys.insert(Key::new(code));
    }

    VirtualDeviceBuilder::new()
        .expect("failed to create virtual device builder")
        .name("umlauter-virtual-keyboard")
        .with_keys(&keys)
        .expect("failed to set keys")
        .build()
        .expect("failed to build virtual device")
}

fn emit_key(vdev: &mut VirtualDevice, key: Key, value: i32) {
    let ev = InputEvent::new(EventType::KEY, key.code(), value);
    let syn = InputEvent::new(EventType::SYNCHRONIZATION, 0, 0);
    vdev.emit(&[ev, syn]).ok();
}

fn emit_backspaces(vdev: &mut VirtualDevice, count: usize) {
    for _ in 0..count {
        emit_key(vdev, Key::KEY_BACKSPACE, 1);
        emit_key(vdev, Key::KEY_BACKSPACE, 0);
        std::thread::sleep(Duration::from_millis(8));
    }
}

fn type_string_xdotool(text: &str) {
    let status = std::process::Command::new("xdotool")
        .arg("type")
        .arg("--clearmodifiers")
        .arg("--delay")
        .arg("12")
        .arg(text)
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => eprintln!("xdotool exited with: {}", s),
        Err(e) => eprintln!("failed to run xdotool: {}", e),
    }
}

impl KeyboardListener {
    pub fn new(
        mappings: HashMap<String, String>,
        max_trigger_len: usize,
        enabled: Arc<AtomicBool>,
    ) -> Self {
        Self {
            mappings,
            max_trigger_len,
            enabled,
        }
    }

    pub fn run(&self) {
        let devices = find_keyboard_devices();
        if devices.is_empty() {
            eprintln!("no keyboard devices found - do you have permission to read /dev/input?");
            eprintln!("try running with sudo or adding your user to the 'input' group");
            return;
        }

        eprintln!("monitoring {} keyboard device(s)", devices.len());

        let mappings = self.mappings.clone();
        let max_len = self.max_trigger_len;
        let enabled = self.enabled.clone();

        let (tx, rx) = mpsc::channel::<KeyEvent>();

        for (dev_name, mut device) in devices {
            let tx = tx.clone();
            std::thread::spawn(move || loop {
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            if event.event_type() != EventType::KEY {
                                continue;
                            }
                            let key = Key::new(event.code());
                            let _ = tx.send(KeyEvent {
                                key,
                                value: event.value(),
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("error reading {}: {}", dev_name, e);
                        std::thread::sleep(Duration::from_secs(1));
                    }
                }
            });
        }

        drop(tx);

        std::thread::spawn(move || {
            Self::process_events(rx, mappings, max_len, enabled);
        });
    }

    fn process_events(
        rx: mpsc::Receiver<KeyEvent>,
        mappings: HashMap<String, String>,
        max_len: usize,
        enabled: Arc<AtomicBool>,
    ) {
        let mut vdev = create_virtual_keyboard();
        std::thread::sleep(Duration::from_millis(200));

        let mut buffer = String::new();
        let mut shift_held = false;
        let mut caps_lock_on = false;

        for event in rx {
            if is_shift(event.key) {
                shift_held = event.value != 0;
                continue;
            }

            if event.key == Key::KEY_CAPSLOCK && event.value == 1 {
                caps_lock_on = !caps_lock_on;
                continue;
            }

            if event.value != 1 {
                continue;
            }

            if !enabled.load(Ordering::Relaxed) {
                buffer.clear();
                continue;
            }

            let Some(ch) = keycode_to_char(event.key, shift_held ^ caps_lock_on) else {
                buffer.clear();
                continue;
            };

            buffer.push(ch);
            if buffer.len() > max_len {
                let excess = buffer.len() - max_len;
                buffer.drain(..excess);
            }

            eprintln!("  buffer: {:?}", buffer);

            let mut matched_trigger: Option<String> = None;
            let mut matched_replacement: Option<String> = None;

            for trigger_len in 2..=buffer.len() {
                let suffix = &buffer[buffer.len() - trigger_len..];
                let suffix_lower = suffix.to_lowercase();
                if let Some(replacement) = mappings.get(&suffix_lower) {
                    let first_char_upper = suffix.chars().next().is_some_and(|c| c.is_uppercase());
                    let replacement = if first_char_upper {
                        replacement.to_uppercase()
                    } else {
                        replacement.clone()
                    };
                    matched_trigger = Some(suffix.to_string());
                    matched_replacement = Some(replacement);
                }
            }

            if let (Some(trigger), Some(replacement)) = (matched_trigger, matched_replacement) {
                eprintln!("  match: {:?} -> {:?}", trigger, replacement);

                if shift_held {
                    emit_key(&mut vdev, Key::KEY_LEFTSHIFT, 0);
                    std::thread::sleep(Duration::from_millis(8));
                }

                let backspace_count = trigger.len();
                emit_backspaces(&mut vdev, backspace_count);
                std::thread::sleep(Duration::from_millis(20));
                type_string_xdotool(&replacement);

                if shift_held {
                    std::thread::sleep(Duration::from_millis(8));
                    emit_key(&mut vdev, Key::KEY_LEFTSHIFT, 1);
                }

                buffer.clear();
            }
        }
    }
}
