# footswitch keyboard edition

Technical details about the keyboard version of the `footswitch` code.

## Hardware support

This version of the code will only work on Arduino (and clones) that support the [USB Keyboard Library][keyboard].

This means you need an Arduino with one of these processors:

* ATmega32U4
* SAMD

## USB

This firmware acts as _both_ a USB HID keyboard _and_ a USB CDC serial device.

The serial device implements the same protocol as [the serial version of the code](../serial/), so you can use [its client](../client/) for microphone control (if desired).

## Keyboard

This code uses the <kbd>F13</kbd> key. We use this key because:

* it's not on most keyboards, so it's unlikely that you'll be using that key for anything else
* it's still supported on Linux, macOS and Windows
* it _isn't_ a modifier key (like `Shift`), so it won't mess up your keybindings (particularly an issue for RPGs)

You could set this to other keys by modifying the 

[keyboard]: https://www.arduino.cc/reference/en/language/functions/usb/keyboard/
