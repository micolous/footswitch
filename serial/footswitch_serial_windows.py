#!/usr/bin/env python3
"""
footswitch_client
Copyright 2021 Michael Farrell <micolous+git@gmail.com>

SPDX-License-Identifier: Apache-2.0

This listens to a serial device for two possible bytes:

  0: the footswitch is released
  1: the footswitch is pressed

The hardware side of this uses an Arduino Nano to report on the state of
shorting one of it's digital GPIO pins to GND, with some de-bounce logic in
there.

This software only runs on Windows.  Sorry :(

This listens to the serial device, and will press a key (F13, set in KEY
constant) while the footswitch is held. You set this in something like Discord
as your push-to-talk key.

If you also have pycaw with a small patch (still working on upstreaming this),
it will also control a microphone device of your choice (MICROPHONE constant).
This has some additional, slower de-bounce logic so that it only mutes your
microphone HOLD_TIME_SEC seconds after you release the footswitch.

On the Audio-Technica USB microphone, it constantly emits a monitor output on
the headphone jack which follows the mute state of the microphone level. When
that's muted, the LED turns amber, and you can't hear yourself anymore through
the monitor output.  When it's unmuted, the LED turns green, and you can hear
yourself once again.

Having discrete microphone control means:

1. Apps that don't support push-to-talk now get push-to-talk!
2. You won't hear yourself on the monitor output whenever you're not holding
   the footswitch, so aren't talking without PTT pressed.

If you don't have a microphone with monitor output, or don't have pycaw, you
don't need this -- it's all optional.

"""

import sched
import serial
import threading
import time
import win32api
import win32con

try:
    from pycaw import pycaw
except ImportError:
    pycaw = None


MICROPHONE = 'AT USB Microphone'
PORT = 'hwgrep://USB-SERIAL CH340'
KEY = win32con.VK_F13
HOLD_TIME_SEC = .25   # for microphone mute only


class MicController:
    """Controller for the microphone device.
    
    This operates a background thread to de-bounce mute events. This lets you
    release the footswitch, and it doesn't actually mute until HOLD_TIME_SEC
    seconds have passed.
    
    If the footswitch is pressed again before the release timer, the scheduled
    mute operation is aborted.
    """

    def __init__(self):
        self._sched = sched.scheduler()
        self.keep_running = True
        self.thread = None
        self._release_at = 0
        self._dev = None
        if pycaw is None or not MICROPHONE:
            # Pycaw not available, just do nothing...
            print('Microphone support disabled.')
            return

        mic_found = False
        for device in pycaw.AudioUtilities.GetAllDevices():
            name = device.FriendlyName
            if name is None:
                continue
            if name.startswith(MICROPHONE):
                print(f'Controlling microphone: {name}')
                self._dev = device
                break

        if self._dev is None:
            print(f'Microphone not found: {MICROPHONE}')

    def unmute(self):
        """Unmutes the microphone, and cancels any pending mute operation."""
        self._release_at = 0
        self._set_device_mute(False)

    def mute(self):
        """Mutes the microphone after HOLD_TIME_SEC seconds."""
        release_at = time.monotonic_ns()
        self._sched.enter(HOLD_TIME_SEC, 1, self._actual_mute, (release_at,))
        self._release_at = release_at

    def _actual_mute(self, release_at):
        if self._release_at == release_at:
            self._release_at = 0
            self._set_device_mute(True)

    def _set_device_mute(self, state):
        if self._dev is None:
            return
        try:
            self._dev.EndpointVolume.SetMute(state, None)
        except Exception as e:
            print(f'Error setting mute state: {e}')

    def _worker(self):
        while self.keep_running:
            self._sched.run()
            time.sleep(HOLD_TIME_SEC)

    def start(self):
        """Spawns a thread to de-bounce mute operations."""
        if self._dev is None:
            return
        self.thread = threading.Thread(target=self._worker)
        self.thread.start()

    def stop(self):
        """Stops the thread that handles de-bounces mute operations."""
        if self.thread is None:
            return
        self.keep_running = False
        self.thread.join()
        self.thread = None


def pumpit():
    s = None
    mic_controller = MicController()
    mic_controller.start()
    try:
        s = serial.serial_for_url(PORT, timeout=1)
        print('Waiting for events...')

        # wait for events...
        while True:
            m = s.read(1)
            if m == b'':
                # no event, keep waiting...
                continue
            elif m == b'0':
                # release
                win32api.keybd_event(KEY, 0, win32con.KEYEVENTF_KEYUP, 0)
                mic_controller.mute()
            elif m == b'1':
                # press
                win32api.keybd_event(KEY, 0, 0, 0)
                mic_controller.unmute()
            else:
                # unexpected input
                print(f'Unexpected input: 0x{m[0]:02x}')
                break
    finally:
        if s is not None:
            s.close()
        mic_controller.stop()


def main():
    while True:
        try:
            pumpit()
        except KeyboardInterrupt:
            raise
        except Exception as e:
            # ignore
            print(f'Error: {e}')
            time.sleep(1.)


if __name__ == '__main__':
    main()
