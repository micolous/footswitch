extern crate serial;

use std::env;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serial::prelude::*;

pub struct MicController {
    chan: mpsc::Receiver<bool>,
}

impl MicController {
    pub fn new(chan: mpsc::Receiver<bool>) -> MicController {
        MicController {
            chan: chan,
        }
    }
    
    pub fn pumpit(&mut self) {
        loop {
            // TODO: set timeout to be sensible?
            let res = self.chan.recv_timeout(Duration::from_millis(1000));
            match res {
                Ok(msg) => {
                    println!("got channel message: {}", msg);
                    
                    // TODO: a thing
                },
                Err(error) => match error {
                    mpsc::RecvTimeoutError::Timeout => continue,
                    _ => panic!("recv error: {}", error),
                }
            }
            
            // TODO: other stuff
        }
    }
}

fn setup<T: SerialPort>(port: &mut T) -> io::Result<()> {
    // 9600 8N1
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud9600)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        Ok(())
    })?;
    
    port.set_timeout(Duration::from_millis(1000))?;
    Ok(())
}

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
    let (tx, rx) = mpsc::channel();
    println!("Hello, world!");

    for arg in env::args_os().skip(1) {
        println!("Port: {:?}", &arg);
        let mut port = serial::open(&arg).unwrap();
        setup(&mut port).unwrap();
        
        let serial_thread = thread::spawn(move || {
            interact(&mut port, tx).unwrap();
        });

        println!("wait for thread");
        
        let mut mc = MicController::new(rx);
        let mic_thread = thread::spawn(move || {
            mc.pumpit();
        });
        
        serial_thread.join().unwrap();
        mic_thread.join().unwrap();
        return;
    }
}
