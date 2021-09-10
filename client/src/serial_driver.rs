extern crate serialport;
use serialport::{FlowControl, SerialPort};

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub const CHANNEL_TIMEOUT: Duration = Duration::from_secs(1);
const MISSING_SERIAL_WAIT_TIME: Duration = Duration::from_secs(10);

/// Creates a connection to a serial port.
pub fn create_serial_port(serial_device: &str) -> Result<Box<dyn SerialPort>, ()> {
    serialport::new(serial_device, 9600)
        .flow_control(FlowControl::Hardware)
        .timeout(CHANNEL_TIMEOUT)
        .open()
        .map_err(|e| {
            debug!("Failed to open serial device -> {:?}", e);
        })
}

/// Sends events from the serial port to the channel.
pub fn interact(mut port: Box<dyn SerialPort>, serial_device: String, chan: mpsc::Sender<bool>) {
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

/// Show an error when no serial device was specified.
pub fn no_device_error() {
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
}
