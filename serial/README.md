# footswitch serial edition

Technical details about the serial version of the `footswitch` code.

## USB

This version of the code can use an Arduino that acts as a USB CDC (serial) device, or an Arduino that includes a USB-TTL serial chip (eg: CH341, FT232R, PL2303).

Arduino models that include a separate USB-TTL serial chip typically require extra device drivers on macOS and Windows.

## Serial protocol

The Ardiuno code writes a single ASCII character for each event:

Character | Hex    | Event
--------- | ------ | --------------
`0`       | `0x30` | Button release, device start-up
`1`       | `0x31` | Button press

Events will only be sent if the button state changes.

## Client

> **Danger: Here be dragons!**
>
> If you want _simple_, run [the `keyboard` version of the Arduino code](../keyboard/). That doesn't require any extra software to work.
>
> This is intended for advanced use cases only, such as automatic microphone mute control.
>
> The client is currently **incomplete**.  It is being rewritten in Rust, and the Python version will go away once it reaches parity.

The client listens to a serial port running the `keyboard.ino` or `serial.ino` code, and has three jobs:

* debounce button presses

* control the mute state of your default microphone device

  This is useful for microphones that have a monitor output that follows the microphone's mute state – so you won't hear yourself unless the PTT button is held.

* (optionally) send a simulated <kbd>F13</kbd> keypress

  This is only needed for serial-only Arduino devices, and is _disabled by default_.

  This doesn't work with Discord on macOS, because of the way it captures global hotkeys.

### Building the Rust client

**Note:** When this is a bit more stable, there'll be Windows release binaries available.

You'll need to [install a Rust toolchain](https://www.rust-lang.org/tools/install), then run:

```sh
cd serial
cargo build --release
```

This will give you an executable in `./target/release/footswitch_serial` (or `footswitch_serial.exe`).

### Old Python client (Windows-only)

**Deprecated**: This will be deleted in future, once the Rust version reaches parity.

Requirements:

* Python 3.8 or later
* pycaw (plus patches that are only in `develop` branch)
* [pywin32][]

[pywin32]: https://github.com/mhammond/pywin32
