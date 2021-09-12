extern crate core_foundation_sys;
extern crate coreaudio;

use core_foundation_sys::string::{
    kCFStringEncodingUTF8, CFStringGetCString, CFStringGetCStringPtr, CFStringRef,
};
use coreaudio::sys::{
    kAudioDevicePropertyDeviceNameCFString, kAudioDevicePropertyMute,
    kAudioDevicePropertyScopeOutput, kAudioHardwareNoError,
    kAudioHardwarePropertyDefaultInputDevice, kAudioHardwarePropertyDevices,
    kAudioObjectPropertyElementMaster, kAudioObjectPropertyScopeGlobal,
    kAudioObjectPropertyScopeInput, kAudioObjectSystemObject, AudioDeviceID,
    AudioDeviceSetProperty, AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize,
    AudioObjectPropertyAddress,
};
use std::ffi::CStr;
use std::mem;
use std::os::raw::c_char;
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
    fn new() -> Self {
        AudioController {}
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

    /// Get input device by name
    fn get_input_device(&self, name: String) -> Result<Box<dyn AudioInputDeviceTrait>, AudioError> {
        for audio_device_id in get_all_device_ids()? {
            let device = AudioInputDevice { audio_device_id };
            if name == device.name()? {
                return Ok(Box::new(device));
            }
        }

        return Err(AudioError {
            msg: "No such device".to_string(),
        });
    }

    // Gets all input device names.
    fn get_input_device_names(&self) -> Result<Vec<String>, AudioError> {
        let mut device_names: Vec<String> = Vec::new();
        for audio_device_id in get_all_device_ids()? {
            device_names.push((AudioInputDevice { audio_device_id }).name()?);
        }
        Ok(device_names)
    }
}

fn get_all_device_ids() -> Result<Vec<AudioDeviceID>, AudioError> {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDevices,
        mScope: kAudioObjectPropertyScopeInput,
        mElement: kAudioObjectPropertyElementMaster,
    };

    let data_size = 0u32;
    let status = unsafe {
        AudioObjectGetPropertyDataSize(
            kAudioObjectSystemObject,
            &property_address as *const _,
            0,
            null(),
            &data_size as *const _ as *mut _,
        )
    };

    if status != kAudioHardwareNoError as i32 {
        return Err(AudioError {
            msg: format!("Error: 0x{:X}", status),
        });
    }
    let device_count = data_size / mem::size_of::<AudioDeviceID>() as u32;

    let mut device_ids: Vec<AudioDeviceID> = Vec::new();
    device_ids.reserve_exact(device_count as usize);

    let status = unsafe {
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &property_address as *const _,
            0,
            null(),
            &data_size as *const _ as *mut _,
            device_ids.as_mut_ptr() as *mut _,
        )
    };

    if status != kAudioHardwareNoError as i32 {
        return Err(AudioError {
            msg: format!("Error: 0x{:X}", status),
        });
    }

    unsafe {
        device_ids.set_len(device_count as usize);
    }

    Ok(device_ids)
}

impl AudioInputDeviceTrait for AudioInputDevice {
    fn name(&self) -> Result<String, AudioError> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyDeviceNameCFString,
            mScope: kAudioDevicePropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMaster,
        };
        let device_name: CFStringRef = null();
        let data_size = mem::size_of::<CFStringRef>();
        let c_str = unsafe {
            try_cf!(AudioObjectGetPropertyData(
                self.audio_device_id,
                &property_address as *const _,
                0,
                null(),
                &data_size as *const _ as *mut _,
                &device_name as *const _ as *mut _,
            ));

            let c_string: *const c_char = CFStringGetCStringPtr(device_name, kCFStringEncodingUTF8);
            if c_string.is_null() {
                // The name could not be returned "efficiently", make a new buffer to try.
                // https://developer.apple.com/documentation/corefoundation/1542133-cfstringgetcstringptr
                let mut buf: [i8; 255] = [0; 255];
                let result = CFStringGetCString(
                    device_name,
                    buf.as_mut_ptr(),
                    buf.len() as _,
                    kCFStringEncodingUTF8,
                );
                if result == 0 {
                    return Err(AudioError {
                        msg: "CFStringGetCString failed to return device name string".to_string(),
                    });
                }
                CStr::from_ptr(buf.as_ptr())
            } else {
                CStr::from_ptr(c_string as *mut _)
            }
        };
        Ok(c_str.to_string_lossy().into_owned())
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
