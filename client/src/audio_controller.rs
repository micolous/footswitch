use std::fmt;
use std::fmt::{Debug, Formatter};
use std::result::Result;

#[derive(Debug)]
pub struct AudioError {
    pub msg: String,
}

/// Trait that describes an audio input device.
pub trait AudioInputDeviceTrait {
    /// The human-readable name of the audio device.
    fn name(&self) -> Result<String, AudioError>;

    /// Sets the mute state of the audio device.
    fn set_mute(&self, state: bool) -> Result<bool, AudioError>;
}

/// Trait that describes the audio subsystem.
pub trait AudioControllerTrait {
    /// Create a new connection to the audio subsystem.
    #[allow(clippy::new_ret_no_self)]
    fn new() -> Self;

    /// Gets the default communications device.
    fn get_comms_device(&self) -> Result<Box<dyn AudioInputDeviceTrait>, AudioError>;

    /// Get input device by name
    fn get_input_device(&self, name: String) -> Result<Box<dyn AudioInputDeviceTrait>, AudioError>;

    // Gets all input device names.
    fn get_input_device_names(&self) -> Result<Vec<String>, AudioError>;
}

impl Debug for dyn AudioInputDeviceTrait {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("AudioInputDevice")
            .field("name", &self.name().unwrap())
            .finish()
    }
}
