extern crate core_foundation_sys;
extern crate coreaudio;

use core_foundation_sys::string::{kCFStringEncodingUTF8, CFStringGetCString, CFStringRef};
use coreaudio::sys::{
    kAudioDevicePropertyDeviceNameCFString, kAudioDevicePropertyMute,
    kAudioDevicePropertyScopeOutput, kAudioHardwareNoError,
    kAudioHardwarePropertyDefaultInputDevice, kAudioObjectPropertyElementMaster,
    kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject, AudioDeviceID,
    AudioDeviceSetProperty, AudioObjectGetPropertyData, AudioObjectPropertyAddress,
};
use std::ffi::CStr;
use std::mem;
use std::ptr::null;

use crate::audio_controller::{AudioControllerTrait, AudioError, AudioInputDeviceTrait};

#[macro_export]
macro_rules! EXAMPLE_PORT {
    () => {
        "/dev/tty.usbmodemHIDPC1"
    };
}

pub struct AudioController {}

pub struct AudioInputDevice {
    audio_device_id: AudioDeviceID,
}

macro_rules! try_cf {
    ($expr:expr) => {
        #[allow(non_upper_case_globals)]
        match $expr as u32 {
            kAudioHardwareNoError => (),
            _ => {
                return Err(AudioError {
                    msg: format!(
                        "Error: {}",
                        coreaudio::Error::from_os_status($expr).err().unwrap()
                    ),
                })
            }
        }
    };
}

// Implementation largely copied from cpal

impl AudioControllerTrait for AudioController {
    fn new() -> Box<dyn AudioControllerTrait> {
        Box::new(AudioController {})
    }

    fn get_comms_device(&self) -> Result<Box<dyn AudioInputDeviceTrait>, AudioError> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDefaultInputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMaster,
        };

        let audio_device_id: AudioDeviceID = 0;
        let data_size = mem::size_of::<AudioDeviceID>();
        let status = unsafe {
            AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
                &property_address as *const _,
                0,
                null(),
                &data_size as *const _ as *mut _,
                &audio_device_id as *const _ as *mut _,
            )
        };
        if status != kAudioHardwareNoError as i32 {
            return Err(AudioError {
                msg: format!("Error: 0x{:X}", status),
            });
        }

        Ok(Box::new(AudioInputDevice { audio_device_id }))
    }
}

impl AudioInputDeviceTrait for AudioInputDevice {
    fn name(&self) -> Result<String, AudioError> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyDeviceNameCFString,
            mScope: kAudioDevicePropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMaster,
        };
        let mut buf: [u8; 255] = [0; 255];
        unsafe {
            let device_name: CFStringRef = null();
            let data_size = mem::size_of::<CFStringRef>();
            try_cf!(AudioObjectGetPropertyData(
                self.audio_device_id,
                &property_address as *const _,
                0,
                null(),
                &data_size as *const _ as *mut _,
                &device_name as *const _ as *mut _,
            ));

            // We could use CFStringGetCStringPtr here first for an "efficient"
            // reference, but this has lifetime issues.
            // https://developer.apple.com/documentation/corefoundation/1542133-cfstringgetcstringptr
            if CFStringGetCString(
                device_name,
                buf.as_mut_ptr() as *mut i8,
                buf.len() as _,
                kCFStringEncodingUTF8,
            ) == 0
            {
                return Err(AudioError {
                    msg: "CFStringGetCString failed to return device name string".to_string(),
                });
            }
        };
        CStr::from_bytes_with_nul(&buf)
            .map_err(|e| AudioError {
                msg: format!("Bad audio device name: {}", e),
            })
            .and_then(|r| Ok(r.to_string_lossy().into_owned()))
    }

    fn set_mute(&self, state: bool) -> Result<bool, AudioError> {
        let cf_state = state as u32;
        let data_size = mem::size_of::<u32>() as u32;
        unsafe {
            try_cf!(AudioDeviceSetProperty(
                self.audio_device_id,
                /* when */ null(),
                /* channel */ 0,
                /* is_input */ 1,
                kAudioDevicePropertyMute,
                data_size,
                &cf_state as *const _ as _,
            ));
        }

        Ok(state)
    }
}
