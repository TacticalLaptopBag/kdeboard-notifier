mod config;
mod setup;
mod tray;

fn main() {
    let configure_only = std::env::args().any(|a| a == "--configure");

    if configure_only || config::Config::load().is_none() {
        setup::run().expect("setup UI failed");
        if configure_only {
            return;
        }
    }

    let Some(config) = config::Config::load() else {
        return; // user cancelled setup with no prior config
    };
    tray::run(config);
}
