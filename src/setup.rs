use iced::{
    widget::{button, column, container, radio, scrollable, text, Column, Row},
    Element, Length, Task,
};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct UsbDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub label: String,
}

#[derive(Debug)]
pub struct SetupApp {
    devices: Vec<UsbDevice>,
    selected: Option<usize>,
    /// Index of the device that matches the on-disk config, if any.
    /// Used to suppress Save when nothing has changed.
    saved_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Select(usize),
    Refresh,
    Save,
    Cancel,
}

pub fn run() -> iced::Result {
    let kde = crate::kde_theme::load();
    let theme = kde.theme;
    iced::application("kdeboard-notifier - Select Keyboard", SetupApp::update, SetupApp::view)
        .theme(move |_| theme.clone())
        .default_font(kde.font)
        .run_with(SetupApp::new)
}

impl SetupApp {
    fn new() -> (Self, Task<Message>) {
        let devices = list_usb_devices();
        let saved_index = Config::load().and_then(|cfg| {
            devices
                .iter()
                .position(|d| d.vendor_id == cfg.vendor_id && d.product_id == cfg.product_id)
        });
        let app = SetupApp { devices, selected: saved_index, saved_index };
        (app, Task::none())
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Select(i) => {
                self.selected = Some(i);
                Task::none()
            }
            Message::Refresh => {
                let prev_selected = self.selected
                    .map(|i| (self.devices[i].vendor_id, self.devices[i].product_id));

                self.devices = list_usb_devices();

                self.saved_index = Config::load().and_then(|cfg| {
                    self.devices.iter().position(|d| {
                        d.vendor_id == cfg.vendor_id && d.product_id == cfg.product_id
                    })
                });

                self.selected = prev_selected
                    .and_then(|(vid, pid)| {
                        self.devices.iter().position(|d| d.vendor_id == vid && d.product_id == pid)
                    })
                    .or(self.saved_index);

                Task::none()
            }
            Message::Save => {
                if let Some(i) = self.selected {
                    let dev = &self.devices[i];
                    Config {
                        vendor_id: dev.vendor_id,
                        product_id: dev.product_id,
                        description: dev.label.clone(),
                    }
                    .save()
                    .expect("failed to save config");
                }
                iced::exit()
            }
            Message::Cancel => iced::exit(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let header = text("Select your keyboard from the connected USB devices below.").size(14);

        let device_rows: Vec<Element<Message>> = if self.devices.is_empty() {
            vec![text("No USB devices found. Ensure the keyboard is connected and try again.").into()]
        } else {
            self.devices
                .iter()
                .enumerate()
                .map(|(i, dev)| {
                    radio(dev.label.as_str(), i, self.selected, Message::Select)
                        .size(16)
                        .into()
                })
                .collect()
        };

        let device_list = scrollable(
            Column::from_vec(device_rows).spacing(8).padding(4),
        )
        .height(Length::Fill);

        let cancel_btn = button(text("Cancel")).on_press(Message::Cancel);
        let save_enabled = self.selected.is_some() && self.selected != self.saved_index;
        let save_btn = if save_enabled {
            button(text("Save")).on_press(Message::Save)
        } else {
            button(text("Save"))
        };

        let actions = Row::new()
            .push(button(text("Refresh Devices")).on_press(Message::Refresh))
            .push(iced::widget::horizontal_space())
            .push(cancel_btn)
            .push(save_btn)
            .spacing(8)
            .align_y(iced::Alignment::Center);

        container(
            column![header, device_list, actions]
                .spacing(16)
                .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn list_usb_devices() -> Vec<UsbDevice> {
    // The kernel exposes manufacturer/product strings in sysfs without requiring
    // the device to be opened, so no udev rules or elevated permissions are needed.
    let Ok(entries) = std::fs::read_dir("/sys/bus/usb/devices") else {
        return vec![];
    };

    let mut result = Vec::new();

    for entry in entries.flatten() {
        let base = entry.path();

        let read = |name| -> Option<String> {
            std::fs::read_to_string(base.join(name))
                .ok()
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
        };

        let Some(vendor_str) = read("idVendor") else { continue };
        let Some(product_str) = read("idProduct") else { continue };

        let Ok(vendor_id) = u16::from_str_radix(&vendor_str, 16) else { continue };
        let Ok(product_id) = u16::from_str_radix(&product_str, 16) else { continue };

        let manufacturer = read("manufacturer").unwrap_or_default();
        let product = read("product")
            .unwrap_or_else(|| format!("{vendor_id:04x}:{product_id:04x}"));

        let label = if manufacturer.is_empty() {
            product
        } else {
            format!("{manufacturer} {product}")
        };

        result.push(UsbDevice { vendor_id, product_id, label });
    }

    result.sort_by(|a, b| a.label.cmp(&b.label));
    result
}
