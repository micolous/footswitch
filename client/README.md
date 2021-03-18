# footswitch client

> **Danger: Here be dragons!**
>
> If you want _simple_, run [the `keyboard` version of the Arduino code](../keyboard/). That doesn't require any extra software to work.

The client listens to a serial port running the `keyboard.ino` or `serial.ino` code, and has three jobs:

* debounce button presses

* control the mute state of your default microphone device

  This is useful for microphones that have a monitor output that follows the microphone's mute state – so you won't hear yourself unless the PTT button is held.

* (optionally) send a simulated <kbd>F13</kbd> keypress

  This is only needed for serial-only Arduino devices, and is _disabled by default_.

  **This doesn't work with Discord on macOS**, because of the way it captures global hotkeys. This is a bug in Discord, and has been reported to them.

## Building the client

**Note:** When this is a bit more stable, there'll be Windows release binaries available.

You'll need to [install a Rust toolchain](https://www.rust-lang.org/tools/install), then run:

```sh
cd client
cargo build --release
```

This will give you an executable in `./target/release/footswitch_serial` (or `footswitch_serial.exe`).

## Known issues

### macOS and simulated keypresses.

Using simulated keypresses (for the serial version) requires access to `Accessibility` APIs (`System Preferences` → `Privacy` → `Accessibility`).

Discord for macOS **does not** support using simulated keypresses to trigger hotkeys. This is because Discord captures global hotkeys in a way that _doesn't_ support accessibility APIs (`IOHIDManager` taps). This is a bug in Discord, and has been reported to them.

Most other applications support simulated keypresses, so will work fine.
