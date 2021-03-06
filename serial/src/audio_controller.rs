use std::fmt::{Debug, Display, Error, Formatter};
use std::result::Result;

pub trait AudioInputDevice {
    fn name(&self) -> String;
    fn set_mute(&self, state: bool);
}

pub trait AudioController {
    fn new() -> Box<dyn AudioController> where Self: Sized;
    fn get_comms_device(&self) -> Result<Box<dyn AudioInputDevice>, String>;
}

impl Display for dyn AudioInputDevice {
    fn fmt(&self, _: &mut Formatter<'_>) -> Result<(), Error> {
        todo!()
    }
}

impl Debug for dyn AudioInputDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("AudioInputDevice")
         .field("name", &self.name())
         .finish()
    }
}
