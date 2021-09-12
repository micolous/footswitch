/*
 * Fake AudioControllerTrait implementation.
 *
 * This is used when the platform is unsupported.
 */

use crate::audio_controller::{AudioControllerTrait, AudioError, AudioInputDeviceTrait};

#[macro_export]
macro_rules! EXAMPLE_PORT {
    () => {
        "/dev/ttyUSB0"
    };
}

pub struct AudioController {
    fake_mic: AudioInputDevice,
}

#[derive(Clone)]
pub struct AudioInputDevice {
    name: String,
}

impl AudioControllerTrait for AudioController {
    fn new() -> Self {
        println!("Using fake audio controller device!");
        AudioController {
            fake_mic: AudioInputDevice {
                name: "Fake Microphone".to_string(),
            },
        }
    }

    fn get_comms_device(&self) -> Result<Box<dyn AudioInputDeviceTrait>, AudioError> {
        Ok(Box::new(self.fake_mic.clone()))
    }

    /// Get input device by name
    fn get_input_device(&self, name: String) -> Result<Box<dyn AudioInputDeviceTrait>, AudioError> {
        if name == self.fake_mic.name {
            Ok(Box::new(self.fake_mic.clone()))
        } else {
            Err(AudioError {
                msg: "device not found".to_string(),
            })
        }
    }

    // Gets all input device names.
    fn get_input_device_names(&self) -> Result<Vec<String>, AudioError> {
        Ok(vec![self.fake_mic.name.clone()])
    }
}

impl AudioInputDeviceTrait for AudioInputDevice {
    fn name(&self) -> Result<String, AudioError> {
        Ok(self.name.clone())
    }

    fn set_mute(&self, state: bool) -> Result<bool, AudioError> {
        Ok(!state)
    }
}
