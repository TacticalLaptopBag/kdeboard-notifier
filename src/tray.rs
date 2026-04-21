use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ksni::{Icon, MenuItem, Tray, TrayService};
use signal_hook::consts::SIGHUP;
use signal_hook::flag as signal_flag;

use crate::config::Config;

struct KeyboardTray {
    connected: bool,
    config: Config,
}

impl Tray for KeyboardTray {
    fn id(&self) -> String {
        "kdeboard-notifier".into()
    }

    fn title(&self) -> String {
        if self.connected {
            format!("{} - connected", self.config.description)
        } else {
            format!("{} - disconnected", self.config.description)
        }
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        vec![status_icon(self.connected)]
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        use ksni::menu::StandardItem;
        vec![StandardItem {
            label: "Quit".into(),
            activate: Box::new(|_| std::process::exit(0)),
            ..Default::default()
        }
        .into()]
    }
}

pub fn run(mut config: Config) -> ! {
    let reload = Arc::new(AtomicBool::new(false));
    signal_flag::register(SIGHUP, reload.clone()).expect("failed to register SIGHUP handler");

    let service = TrayService::new(KeyboardTray {
        connected: false,
        config: config.clone(),
    });
    let handle = service.handle();
    service.spawn();

    loop {
        if reload.swap(false, Ordering::Relaxed) {
            if let Some(new_cfg) = Config::load() {
                config = new_cfg.clone();
                handle.update(|t| t.config = new_cfg);
            }
        }

        let connected = is_device_connected(config.vendor_id, config.product_id);
        handle.update(|t| t.connected = connected);

        std::thread::sleep(Duration::from_secs(1));
    }
}

fn is_device_connected(vendor_id: u16, product_id: u16) -> bool {
    rusb::devices()
        .ok()
        .map(|list| {
            list.iter().any(|d| {
                d.device_descriptor()
                    .map(|desc| desc.vendor_id() == vendor_id && desc.product_id() == product_id)
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn status_icon(connected: bool) -> Icon {
    const SIZE: i32 = 22;
    let (r, g, b): (u8, u8, u8) = if connected { (40, 200, 40) } else { (120, 120, 120) };
    let cx = SIZE / 2;
    let cy = SIZE / 2;
    let radius_sq = (SIZE / 2 - 2).pow(2);

    let mut data = vec![0u8; (SIZE * SIZE * 4) as usize];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= radius_sq {
                let i = ((y * SIZE + x) * 4) as usize;
                data[i] = 255; // A
                data[i + 1] = r; // R
                data[i + 2] = g; // G
                data[i + 3] = b; // B
            }
        }
    }

    Icon { width: SIZE, height: SIZE, data }
}
