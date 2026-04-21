use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use ksni::{Icon, MenuItem, Tray, TrayService};
use signal_hook::consts::SIGHUP;
use signal_hook::flag as signal_flag;

use crate::config::Config;

static CONNECTED_ICONS: LazyLock<Vec<Icon>> = LazyLock::new(|| {
    render_svg_icon(include_bytes!("../assets/keyboard-connected.svg"))
});

static DISCONNECTED_ICONS: LazyLock<Vec<Icon>> = LazyLock::new(|| {
    render_svg_icon(include_bytes!("../assets/keyboard-disconnected.svg"))
});

fn render_svg_icon(data: &[u8]) -> Vec<Icon> {
    [16u32, 22].map(|size| render_at(data, size)).to_vec()
}

fn render_at(data: &[u8], size: u32) -> Icon {
    use resvg::{tiny_skia, usvg};

    let tree = usvg::Tree::from_data(data, &usvg::Options::default())
        .expect("invalid SVG");

    let svg = tree.size();
    let scale = size as f32 / svg.width().max(svg.height());
    let mut pixmap = tiny_skia::Pixmap::new(size, size).expect("zero-size pixmap");
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());

    // tiny-skia pixels are RGBA premultiplied; SNI wants ARGB32 straight alpha.
    let argb: Vec<u8> = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let a = p.alpha();
            let (r, g, b) = if a == 0 {
                (0, 0, 0)
            } else {
                let af = a as u32;
                (
                    (p.red() as u32 * 255 / af) as u8,
                    (p.green() as u32 * 255 / af) as u8,
                    (p.blue() as u32 * 255 / af) as u8,
                )
            };
            [a, r, g, b]
        })
        .collect();

    Icon { width: size as i32, height: size as i32, data: argb }
}

enum TrayEvent {
    Configure,
}

struct KeyboardTray {
    connected: bool,
    config: Config,
    tx: Sender<TrayEvent>,
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
        if self.connected {
            CONNECTED_ICONS.clone()
        } else {
            DISCONNECTED_ICONS.clone()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(TrayEvent::Configure);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        use ksni::menu::StandardItem;
        let tx = self.tx.clone();
        vec![
            StandardItem {
                label: "Configure...".into(),
                activate: Box::new(move |_| {
                    let _ = tx.send(TrayEvent::Configure);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn run(mut config: Config) -> ! {
    let reload = Arc::new(AtomicBool::new(false));
    signal_flag::register(SIGHUP, reload.clone()).expect("failed to register SIGHUP handler");

    let (tx, rx): (Sender<TrayEvent>, Receiver<TrayEvent>) = mpsc::channel();

    let service = TrayService::new(KeyboardTray {
        connected: false,
        config: config.clone(),
        tx,
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

        if let Ok(TrayEvent::Configure) = rx.try_recv() {
            let exe = std::env::current_exe().expect("could not find current exe");
            let _ = std::process::Command::new(exe).arg("--configure").spawn().and_then(|mut c| c.wait());
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
    let target_vendor = format!("{:04x}", vendor_id);
    let target_product = format!("{:04x}", product_id);
    let Ok(entries) = std::fs::read_dir("/sys/bus/usb/devices") else {
        return false;
    };
    entries.flatten().any(|e| {
        let p = e.path();
        let v = std::fs::read_to_string(p.join("idVendor")).unwrap_or_default();
        let d = std::fs::read_to_string(p.join("idProduct")).unwrap_or_default();
        v.trim() == target_vendor && d.trim() == target_product
    })
}

