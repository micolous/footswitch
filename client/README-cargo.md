# footswitch client

This is a Rust client for [a USB footswitch][footswitch] for push-to-talk in voice chat apps.

![Footswitch built on the Pro Micro](https://github.com/micolous/footswitch/raw/main/images/pro-micro-footswitch.jpg)

This client turns serial events from the footswitch into a synthetic keystroke for activating push-to-talk, and automatically mutes and unmutes the microphone.

Most people shouldn't need this, and can use the keyboard version of the code on their Arduino which acts as a USB HID keyboard.

More information about the project, and instructions on how to build your own footswitch, are available [from the project's website][footswitch].

[footswitch]: https://github.com/micolous/footswitch
