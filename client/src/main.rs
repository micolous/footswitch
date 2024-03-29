#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[cfg(feature = "enigo")]
extern crate enigo;
extern crate serialport;

use std::cmp::{max, min};
use std::io;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(feature = "enigo")]
use enigo::{Enigo, Key, KeyboardControllable};
use serialport::{FlowControl, SerialPort};

mod audio_controller;
use audio_controller::{AudioControllerTrait, AudioError, AudioInputDeviceTrait};

#[macro_use]
#[cfg_attr(target_os = "macos", path = "macos.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod os;
use os::AudioController;

#[cfg(feature = "enigo")]
const KEYCODE: Key = Key::F13;
const CHANNEL_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_DEBOUNCE: Duration = Duration::from_secs(10);
const DEFAULT_DEBOUNCE: Duration = Duration::from_millis(100);
const MISSING_SERIAL_WAIT_TIME: Duration = Duration::from_secs(10);

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
    comms_device: Option<&'a dyn AudioInputDeviceTrait>,
    #[cfg(feature = "enigo")]
    enigo: Option<Enigo>,
    debounce: Duration,
    controller_state: ControllerState,
}

impl MicController<'_> {
    pub fn new<T: AudioControllerTrait>(
        chan: mpsc::Receiver<bool>,
        #[cfg(feature = "enigo")] keyboard_emulation: bool,
        microphone_control: bool,
        debounce: Duration,
    ) -> Self {
        MicController {
            chan,
            comms_device: if microphone_control {
                let audio = Box::leak(T::new());
                Some(Box::leak(audio.get_comms_device().unwrap()))
            } else {
                None
            },
            #[cfg(feature = "enigo")]
            enigo: if keyboard_emulation {
                Some(Enigo::new())
            } else {
                None
            },
            debounce,
            controller_state: ControllerState::Released,
        }
    }

    pub fn device_name(&self) -> Result<String, AudioError> {
        match self.comms_device {
            Some(c) => c.name(),
            None => Ok("None".to_string()),
        }
    }

    fn dispatch(&mut self) -> Result<(), AudioError> {
        match self.controller_state {
            ControllerState::Pressed => {
                debug!("Button pressing");
                self.controller_state = ControllerState::Held;
                #[cfg(feature = "enigo")]
                if let Some(e) = self.enigo.as_mut() {
                    e.key_up(KEYCODE)
                };
                return match self.comms_device {
                    Some(c) => c.set_mute(false).map(|_| ()),
                    None => Ok(()),
                };
            }
            ControllerState::ReleaseWait(released_at) => {
                if released_at.elapsed() >= self.debounce {
                    debug!("Button releasing");
                    self.controller_state = ControllerState::Released;
                    #[cfg(feature = "enigo")]
                    if let Some(e) = self.enigo.as_mut() {
                        e.key_down(KEYCODE)
                    };
                    return match self.comms_device {
                        Some(c) => c.set_mute(true).map(|_| ()),
                        None => Ok(()),
                    };
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn pumpit(&mut self) -> Result<(), AudioError> {
        loop {
            let res = self.chan.recv_timeout(match self.controller_state {
                ControllerState::ReleaseWait(released_at) => min(
                    CHANNEL_TIMEOUT,
                    max(
                        Duration::from_millis(1),
                        self.debounce - released_at.elapsed(),
                    ),
                ),
                _ => CHANNEL_TIMEOUT,
            });
            match res {
                Ok(msg) => {
                    if msg {
                        match self.controller_state {
                            ControllerState::Released => {
                                self.controller_state = ControllerState::Pressed
                            }
                            ControllerState::ReleaseWait(_) => {
                                self.controller_state = ControllerState::Held
                            }
                            _ => {}
                        }
                    } else {
                        self.controller_state = ControllerState::ReleaseWait(Instant::now());
                    }
                    self.dispatch()?;
                }
                Err(error) => match error {
                    mpsc::RecvTimeoutError::Timeout => {
                        self.dispatch()?;
                    }
                    _ => {
                        // The other side has probably gone away!
                        info!("Closing pumpit thread");
                        return Ok(());
                    }
                },
            }
        }
    }
}

fn create_serial_port(serial_device: &str) -> Result<Box<dyn SerialPort>, serialport::Error> {
    serialport::new(serial_device, 9600)
        .flow_control(FlowControl::Hardware)
        .timeout(CHANNEL_TIMEOUT)
        .open()
        .map_err(|e| {
            error!("Failed to open {}: {}", serial_device, e.description);
            e
        })
}

/// Sends events from the serial port to the channel.
fn interact(mut port: Box<dyn SerialPort>, serial_device: String, chan: mpsc::Sender<bool>) {
    let mut buf = [0; 1];

    'outer: loop {
        'inner: loop {
            let res = port.read(&mut buf[..]);
            match res {
                Ok(len) => {
                    if len == 1 {
                        match chan.send(match buf[0] {
                            b'0' => false,
                            b'1' => true,
                            _ => {
                                warn!("Unhandled serial input: {}", buf[0]);
                                break 'inner;
                            }
                        }) {
                            Ok(()) => {}
                            Err(_) => {
                                // Other end of the channel has probably gone away.
                                // Shut down the thread.
                                break 'outer;
                            }
                        }
                    } else {
                        warn!("Unhandled serial input length ({}): {:?}", len, &buf[..len]);
                        break 'inner;
                    }
                }
                Err(error) => match error.kind() {
                    io::ErrorKind::TimedOut => continue,
                    _ => {
                        warn!("Error reading serial device: {:?}", error);
                        break 'inner;
                    }
                },
            }
        }

        // Something went wrong - reset the serial port if possible.
        port = 'reset: loop {
            match create_serial_port(&serial_device) {
                Ok(p) => {
                    warn!("Reconnecting device {}", &serial_device);
                    break 'reset p;
                }
                Err(_) => thread::sleep(MISSING_SERIAL_WAIT_TIME),
            }
        };
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let port_help = concat!(
        "Port/path of the footswitch's serial device (eg: ",
        EXAMPLE_PORT!(),
        ")"
    );
    let default_debounce = DEFAULT_DEBOUNCE.as_millis().to_string();

    let matches = clap_app!(footswitch =>
        (version: "0.1")
        (author: "Michael Farrell <https://github.com/micolous/footswitch>")
        (about: "Serial control client for a USB footswitch")
        (@arg DEVICE: port_help)
        (@arg keyboard_emulation: -k --keyboard
            "Enables keyboard input emulation; only needed for serial.ino")
        (@arg debounce_duration: -d --debounce
            default_value(&default_debounce)
            value_name("MSEC")
            "Debounce duration, in milliseconds")
        (@arg no_mute: -M --no_mute
            "Disables automatic microphone mute control")
    )
    .get_matches();

    let keyboard_emulation = matches.is_present("keyboard_emulation");
    #[cfg(not(feature = "enigo"))]
    if keyboard_emulation {
        error!("Keyboard input emulation support is not available in this build.");
        return;
    }
    let microphone_control = !matches.is_present("no_mute");

    let serial_device = match matches.value_of("DEVICE") {
        Some(v) => v.to_string(),
        None => {
            error!("No device specified. Available serial ports:");
            let ports = serialport::available_ports().unwrap_or_else(|_| {
                error!("Unable to probe for available serial ports!");
                Vec::with_capacity(0)
            });
            if ports.is_empty() {
                error!("No serial ports found!");
            } else {
                for p in ports {
                    error!("* {}", p.port_name);
                }
            }
            return;
        }
    };

    let debounce_duration = match u64::from_str(
        matches.value_of("debounce_duration").unwrap(), // Default set in clap_app! macro
    )
    .map(Duration::from_millis)
    {
        Err(e) => {
            error!("Error parsing debounce duration: {}", e);
            return;
        }
        Ok(d) => {
            if d > MAX_DEBOUNCE {
                error!(
                    "--debounce must be less than or equal to {} milliseconds",
                    MAX_DEBOUNCE.as_millis()
                );
                return;
            }
            d
        }
    };

    let (tx, rx) = mpsc::channel();

    info!("Serial port: {}", &serial_device);
    #[cfg(feature = "enigo")]
    info!(
        "Keyboard emulation: {}",
        if keyboard_emulation { "on" } else { "off" }
    );
    info!("Debounce: {} ms", debounce_duration.as_millis());
    let port = create_serial_port(&serial_device).expect("Failed to open port");

    let serial_thread = thread::spawn(move || {
        interact(port, serial_device, tx);
    });

    let mut mc = MicController::new::<AudioController>(
        rx,
        #[cfg(feature = "enigo")]
        keyboard_emulation,
        microphone_control,
        debounce_duration,
    );
    if microphone_control {
        info!(
            "Microphone device: {}",
            mc.device_name().unwrap_or_else(|_| "unknown".to_string())
        );
    } else {
        info!("Microphone control disabled.");
    }
    info!("Ready, waiting for footswitch press...");

    match mc.pumpit() {
        Ok(()) => {}
        Err(e) => error!("Error in MicController: {:?}", e),
    }
    drop(mc);

    serial_thread.join().unwrap();
}
