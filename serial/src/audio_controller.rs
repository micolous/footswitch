use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::result::Result;

#[derive(Debug)]
pub struct AudioError {
    pub msg: String,
}

pub trait AudioInputDevice {
    fn name(&self) -> String;
    fn set_mute(&self, state: bool) -> Result<bool, AudioError>;
}

pub trait AudioController {
    fn new() -> Box<dyn AudioController> where Self: Sized;
    fn get_comms_device(&self) -> Result<Box<dyn AudioInputDevice>, AudioError>;
}

impl Display for dyn AudioInputDevice {
    fn fmt(&self, _: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        todo!()
    }
}

impl Debug for dyn AudioInputDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("AudioInputDevice")
         .field("name", &self.name())
         .finish()
    }
}
