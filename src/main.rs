mod config;
mod setup;
mod tray;

fn main() {
    if config::Config::load().is_none() {
        setup::run().expect("setup UI failed");
    }

    let config = config::Config::load().expect("no config found; re-run to reconfigure");
    tray::run(config);
}
