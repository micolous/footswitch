# footswitch client

> **Danger: Here be dragons!**
>
> If you want _simple_, run [the `keyboard` version of the Arduino code](../keyboard/). That doesn't require any extra software to work.

The client listens to a serial port running the `keyboard.ino` or `serial.ino` code, and has three jobs:

* debounce button presses

* control the mute state of a microphone device

  This is useful for microphones that have a monitor output that follows the microphone's mute state – so you won't hear yourself unless the PTT button is held.

* (optionally) send a simulated <kbd>F13</kbd> keypress

  This is only needed for serial-only Arduino devices, and is _disabled by default_.

  **This doesn't work with Discord on macOS**, because of the way it captures global hotkeys. This is a bug in Discord, and has been reported to them.

## Building the client

**Note:** When this is a bit more stable, there'll be Windows release binaries available.

You'll need to [install a Rust toolchain](https://www.rust-lang.org/tools/install), then run:

```sh
cd client
cargo build
```

This will give you an executable in `./target/debug/footswitch` (or `footswitch.exe`).

## Running the client

Run the `footswitch_serial` executable at the command-line with the device's serial port name/path:

* **macOS:**

  ```
  user@host client % cargo run -- /dev/tty.usbmodemHIDPC1
  Serial port: /dev/tty.usbmodemHIDPC1
  Keyboard emulation: off
  Debounce: 100 ms
  Microphone device: MacBook Pro Microphone
  Ready, waiting for footswitch press...
  ```

* **Windows:**

  ```
  D:\footswitch\client> cargo run -- COM3
  Serial port: COM3
  Keyboard emulation: off
  Debounce: 100 ms
  Microphone device: Microphone (High Definition Audio Device)
  Ready, waiting for footswitch press...
  ```

You can stop the client by pressing <kbd>Control</kbd> + <kbd>C</kbd>.

The client takes the following command-line flags (which also can be seen by running `cargo run -- --help`):

* `--keyboard`: Enables keyboard input emulation. This is only needed if you're running [serial.ino](../serial/serial.ino).
* `--no_mute`: Disables automatic microphone mute control.
* `--mic_device <NAME>`: Select which microphone device to control. If not specified, the default communications device is used.
* `--list_mic_devices`: Show a list of available microphone devices, then exit.
* `--debounce <MSEC>`: Number of milliseconds to wait after the footswitch is released before releasing the PTT key and muting the microphone again.

You can also run the client without any command-line arguments to get a list of serial ports on your system:

```
% cargo run --
No device specified. Available serial ports:
* /dev/tty.Bluetooth-Incoming-Port
* /dev/tty.usbmodemHIDPC1
```

## Known issues

### macOS and simulated keypresses.

Using simulated keypresses (for the serial version) requires access to `Accessibility` APIs (`System Preferences` → `Privacy` → `Accessibility`).

Discord for macOS **does not** support using simulated keypresses to trigger hotkeys. This is because Discord captures global hotkeys in a way that _doesn't_ support accessibility APIs (`IOHIDManager` taps). This is a bug in Discord, and has been reported to them.

Most other applications support simulated keypresses, so will work fine.

## Client design

The client runs with two threads:

* The `serial` thread listens to events from the footswitch's serial port, and broadcasts them over [a channel][mpsc] to the `main` thread.

* The `main` thread listens to to events from the `serial` thread, and runs the `MicController` state machine.

The `MicController` state machine is responsible for debouncing incoming events, muting and unmuting the microphone device, and pressing and releasing synthetic key events.

OS-specific audio mixer code implements the `AudioControllerTrait` (`audio_controller.rs`), which has a minimal set of controls each platform needs to expose:

* `macos.rs`: macOS CoreAudio mixer implementation
* `windows.rs`: Windows MMDevice mixer implementation
* `os.rs`: a stub (fake) mixer implementation

In future, the plan is to find a cross-platform audio library that will allow this to stop shipping as much OS-specific code. :)

Synthetic keypress events are handled by `enigo`, including all platform-specific code.

[mpsc]: https://doc.rust-lang.org/std/sync/mpsc/
