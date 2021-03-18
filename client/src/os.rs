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

pub struct AudioController {}

pub struct AudioInputDevice {
    name: String,
}

impl AudioControllerTrait for AudioController {
    fn new() -> Box<dyn AudioControllerTrait> {
        println!("Using fake audio controller device!");
        Box::new(AudioController {})
    }

    fn get_comms_device(&self) -> Result<Box<dyn AudioInputDeviceTrait>, AudioError> {
        Ok(Box::new(AudioInputDevice {
            name: "Fake Microphone".to_string(),
        }))
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
