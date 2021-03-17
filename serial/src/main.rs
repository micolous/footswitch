#[macro_use]
extern crate clap;
extern crate enigo;
extern crate serial;

use std::cmp::{max, min};
use std::io;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use enigo::{Enigo, Key, KeyboardControllable};
use serial::prelude::*;

mod audio_controller;
use audio_controller::{AudioControllerTrait, AudioInputDeviceTrait, AudioError};

#[macro_use]
#[cfg_attr(target_os = "macos", path = "macos.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod os;
use os::AudioController;

const KEYCODE : Key = Key::F13;
const CHANNEL_TIMEOUT : Duration = Duration::from_secs(1);
const MAX_DEBOUNCE : Duration = Duration::from_secs(10);

#[derive(Debug, PartialEq)]
pub enum ControllerState {
    /// The button has been fully released.
    Released,

    /// The button has just been pressed.
    Pressed,

    /// The button is held.
    Held,

    /// The button has been recently released, and is waiting for debounce.
    /// The Instant is when the button was released.
    ReleaseWait(Instant),
}

pub struct MicController<'a> {
    chan: mpsc::Receiver<bool>,
    comms_device: &'a dyn AudioInputDeviceTrait,
    enigo: Option<Enigo>,
    debounce: Duration,
    controller_state: ControllerState,
}

impl MicController<'_> {
    pub fn new<T: AudioControllerTrait>(
        chan: mpsc::Receiver<bool>,
        keyboard_emulation: bool,
        debounce: Duration,
    ) -> Self {
        let audio = Box::leak(T::new());
        let comms_device = Box::leak(audio.get_comms_device().unwrap());

        MicController {
            chan: chan,
            comms_device: comms_device,
            enigo: if keyboard_emulation { Some(Enigo::new()) } else { None },
            debounce: debounce,
            controller_state: ControllerState::Released,
        }
    }

    pub fn device_name(&self) -> Result<String, AudioError> {
        self.comms_device.name()
    }

    fn dispatch(&mut self) {
        match self.controller_state {
            ControllerState::Pressed => {
                println!("Button pressing");
                self.enigo.as_mut().map(|e| e.key_up(KEYCODE));
                self.comms_device.set_mute(false).unwrap();
                self.controller_state = ControllerState::Held;
            },
            ControllerState::ReleaseWait(released_at) => {
                if released_at.elapsed() >= self.debounce {
                    println!("Button releasing");
                    self.controller_state = ControllerState::Released;
                    self.enigo.as_mut().map(|e| e.key_down(KEYCODE));
                    self.comms_device.set_mute(true).unwrap();
                }
            },
            _ => {}
        }
    }

    pub fn pumpit(&mut self) {
        loop {
            let res = self.chan.recv_timeout(match self.controller_state {
                ControllerState::ReleaseWait(released_at) => {
                    min(
                        CHANNEL_TIMEOUT,
                        max(
                            Duration::from_millis(1),
                            self.debounce - released_at.elapsed()))
                },
                _ => CHANNEL_TIMEOUT
            });
            match res {
                Ok(msg) => {
                    if msg {
                        match self.controller_state {
                            ControllerState::Released => self.controller_state = ControllerState::Pressed,
                            ControllerState::ReleaseWait(_) => self.controller_state = ControllerState::Held,
                            _ => {},
                        }
                    } else {
                        self.controller_state = ControllerState::ReleaseWait(Instant::now());
                    }
                    self.dispatch();
                },
                Err(error) => match error {
                    mpsc::RecvTimeoutError::Timeout => {
                        self.dispatch();
                    },
                    _ => panic!("recv error: {}", error),
                }
            }
        }
    }
}

/// Configures the serial port.
fn setup<T: SerialPort>(port: &mut T) -> io::Result<()> {
    // 9600 8N1
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud9600)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })?;

    port.set_timeout(Duration::from_millis(1000))?;
    Ok(())
}

/// Sends events from the serial port to the channel.
fn interact<T: SerialPort>(port: &mut T, chan: mpsc::Sender<bool>) -> io::Result<()> {
    let mut buf = [0; 1];

    loop {
        let res = port.read(&mut buf[..]);
        match res {
            Ok(len) => {
                if len == 1 {
                    chan.send(match buf[0] {
                        b'0' => false,
                        b'1' => true,
                        _ => panic!("unhandled serial input: {}", buf[0])
                    }).expect("channel error");
                } else {
                    panic!("unhandled serial input: {:?}", &buf[..len])
                }
            },
            Err(error) => match error.kind() {
                io::ErrorKind::TimedOut => continue,
                _ => return Err(error)
            }
        }
    }
}

fn main() {
    let port_help = concat!("Port/path of the footswitch's serial device (eg: ", EXAMPLE_PORT!(), ")");
    let matches = clap_app!(footswitch =>
        (version: "0.1")
        (author: "Michael Farrell <https://github.com/micolous/footswitch>")
        (about: "Serial control client for a USB footswitch")
        (@arg DEVICE: +required port_help)
        (@arg keyboard_emulation: -k --keyboard "Enables keyboard input emulation; only needed for serial.ino")
        (@arg debounce_duration: -d --debounce default_value("100") value_name("MSEC") "Debounce duration, in milliseconds")
    ).get_matches();

    let keyboard_emulation = matches.is_present("keyboard_emulation");
    let serial_device = matches.value_of("DEVICE").unwrap();
    let debounce_duration = u64::from_str(matches.value_of("debounce_duration").unwrap()).map(|d| Duration::from_millis(d)).unwrap();
    if debounce_duration > MAX_DEBOUNCE {
        panic!("--debounce must be less than or equal to {} milliseconds", MAX_DEBOUNCE.as_millis());
    }

    let (tx, rx) = mpsc::channel();

    println!("Serial port: {}", serial_device);
    println!("Keyboard emulation: {}", if keyboard_emulation {"on"} else {"off"});
    println!("Debounce: {} ms", debounce_duration.as_millis());
    let mut port = serial::open(serial_device).unwrap();
    setup(&mut port).unwrap();

    let serial_thread = thread::spawn(move || {
        interact(&mut port, tx).unwrap();
    });

    let mut mc = MicController::new::<AudioController>(rx, keyboard_emulation, debounce_duration);
    println!("Microphone device: {}", mc.device_name().unwrap());
    println!("Ready, waiting for footswitch press...");
    mc.pumpit();

    serial_thread.join().unwrap();
}
