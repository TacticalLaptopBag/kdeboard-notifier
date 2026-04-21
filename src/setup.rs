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

#[derive(Debug, Default)]
pub struct SetupApp {
    devices: Vec<UsbDevice>,
    selected: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Select(usize),
    Save,
    Cancel,
}

pub fn run() -> iced::Result {
    iced::application("kdeboard-notifier - Select Keyboard", SetupApp::update, SetupApp::view)
        .run_with(SetupApp::new)
}

impl SetupApp {
    fn new() -> (Self, Task<Message>) {
        let app = SetupApp {
            devices: list_usb_devices(),
            selected: None,
        };
        (app, Task::none())
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Select(i) => {
                self.selected = Some(i);
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
        let save_btn = if self.selected.is_some() {
            button(text("Save")).on_press(Message::Save)
        } else {
            button(text("Save"))
        };

        let actions = Row::new()
            .push(cancel_btn)
            .push(save_btn)
            .spacing(8);

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
    let Ok(devices) = rusb::devices() else {
        return vec![];
    };

    let mut result = Vec::new();

    for device in devices.iter() {
        let Ok(desc) = device.device_descriptor() else {
            continue;
        };

        let vendor_id = desc.vendor_id();
        let product_id = desc.product_id();

        let label = match device.open() {
            Ok(handle) => {
                let timeout = std::time::Duration::from_millis(200);
                let manufacturer = handle
                    .read_manufacturer_string_ascii(&desc)
                    .unwrap_or_default();
                let product = handle
                    .read_product_string_ascii(&desc)
                    .unwrap_or_else(|_| format!("{vendor_id:04x}:{product_id:04x}"));
                let _ = timeout; // suppress unused warning — timeout used implicitly by rusb
                if manufacturer.is_empty() {
                    product
                } else {
                    format!("{manufacturer} {product}")
                }
            }
            Err(_) => format!("{vendor_id:04x}:{product_id:04x}"),
        };

        println!("{vendor_id}:{product_id} - {label}");
        result.push(UsbDevice { vendor_id, product_id, label });
    }

    result
}
