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

When running [the client](../client/), these serial events are turned into synthetic keypress events.
