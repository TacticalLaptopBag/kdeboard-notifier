# KDEBoard Notifier

A basic Rust application that shows a USB device connection status via a system tray icon.
This was meant to easily determine if a USB switch was pointed to a device or not.

NOTE: This application is entirely vibe coded. Since I did not write this code, it will remain unlicensed.
I've read and tested the code enough to understand what it does, but there are probably better ways to do this.
Oh well! :shrug:

## Build

### From Source

* [Install Rust][rustup]
* Run `cargo build --release`
* Executable is in `target/release/kdeboard-notifier`

### Flatpak

Ensure `flatpak` and `flatpak-builder` are installed, then run these commands:
```bash
./generate-flatpak-sources.sh
flatpak --user install -y org.freedesktop.Sdk.Extension.rust-stable//24.08 org.kde.Platform//6.9 org.kde.Sdk//6.9
flatpak-builder --force-clean --repo=repo.flatpak.d build.flatpak.d io.github.tacticallaptopbag.KDEBoardNotifier.yaml
flatpak build-bundle repo.flatpak.d kdeboard-notifier.flatpak io.github.tacticallaptopbag.KDEBoardNotifier
```
This will create a `kdeboard-notifier.flatpak` file, which can be installed with
```bash
flatpak --user install kdeboard-notifier.flatpak
```


<!-- links -->
[rustup]: https://rustup.rs/
