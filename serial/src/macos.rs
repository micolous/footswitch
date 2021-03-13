
use crate::audio_controller::{AudioControllerTrait, AudioInputDeviceTrait, AudioError};

pub struct AudioController {
}

pub struct AudioInputDevice {
    name: String,
}

impl AudioControllerTrait for AudioController {
    fn new() -> Box<dyn AudioControllerTrait> {
        Box::new(AudioController { })
    }
    
    fn get_comms_device(&self) -> Result<Box<dyn AudioInputDeviceTrait>, AudioError> {
        Ok(Box::new(AudioInputDevice { name: "test".to_string() }))
    }
}

impl AudioInputDeviceTrait for AudioInputDevice {
    fn name(&self) -> String {
        self.name.clone()
    }
    
    fn set_mute(&self, state: bool) -> Result<bool, AudioError> {
        Ok(!state)
    }
}
