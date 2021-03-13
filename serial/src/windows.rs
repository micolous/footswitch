extern crate winapi;

use std::convert::TryInto;
use std::ptr::null_mut;
use std::mem;
use std::result::Result;
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::slice;

use crate::audio_controller::{AudioControllerTrait, AudioInputDeviceTrait, AudioError};

use winapi::{
    um::{
        combaseapi::{CoCreateInstance, CLSCTX_ALL},
        coml2api::STGM_READ,
        endpointvolume::IAudioEndpointVolume,
        functiondiscoverykeys_devpkey::PKEY_Device_FriendlyName,
        mmdeviceapi::{
            CLSID_MMDeviceEnumerator,
            IMMDevice,
            IMMDeviceEnumerator,
            eCapture,
            eCommunications,
        },
        objbase::CoInitialize,
        propkeydef::REFPROPERTYKEY,
        propsys::IPropertyStore,
        winbase::lstrlenW,
    },
    shared::{
        minwindef::BOOL,
        winerror::{S_OK, S_FALSE},
        wtypesbase::{CLSCTX_INPROC_SERVER}
    },
    Interface,
};

macro_rules! try_com {
	($expr:expr) => (
        match $expr {
            S_OK => true,
            S_FALSE => false,
            _ => return Err(AudioError{ msg: format!("HRESULT: 0x{:X}", $expr) }),
		}
    )
}

pub struct AudioController {
}


pub struct AudioInputDevice {
    mm_device: *mut IMMDevice,
    name: String,
    audio_endpoint_volume: *mut IAudioEndpointVolume,
}

impl AudioController {   
    fn get_device_enumerator(&self) -> Result<*mut IMMDeviceEnumerator, AudioError> {
        unsafe {
            let mut device_enumerator = mem::MaybeUninit::<&mut IMMDeviceEnumerator>::uninit();
            try_com!(CoCreateInstance(
                &CLSID_MMDeviceEnumerator,
                null_mut(),
                CLSCTX_INPROC_SERVER,
                &IMMDeviceEnumerator::uuidof() as *const _,
                device_enumerator.as_mut_ptr() as *mut *mut _));
            Ok(device_enumerator.assume_init())
        }
    }

    fn get_default_communications_imm_device(&self, device_enumerator: *mut IMMDeviceEnumerator) -> Result<*mut IMMDevice, AudioError> {
        unsafe {
            let mut mm_device = mem::MaybeUninit::<>::uninit();

            try_com!((*device_enumerator).GetDefaultAudioEndpoint(
                eCapture,
                eCommunications,
                mm_device.as_mut_ptr()));
            Ok(mm_device.assume_init())
        }
    }
}


impl AudioControllerTrait for AudioController {
    fn new() -> Box<dyn AudioControllerTrait> {
        unsafe {
            CoInitialize(null_mut());
        }
        Box::new(AudioController { })
    }
    
    fn get_comms_device(&self) -> Result<Box<dyn AudioInputDeviceTrait>, AudioError> {
        let device_enumerator = self.get_device_enumerator()?;

        Ok(Box::new(AudioInputDevice::new(self.get_default_communications_imm_device(device_enumerator)?)?))
    }
}

impl AudioInputDevice {
    fn new(mm_device: *mut IMMDevice) -> Result<WindowsAudioInputDevice, AudioError> {
        // Read properties
        let props = AudioInputDevice::open_property_store(mm_device)?;
        
        Ok(AudioInputDevice {
            mm_device: mm_device,
            name: WindowsAudioInputDevice::get_property_value(props, &PKEY_Device_FriendlyName)?,
            audio_endpoint_volume: WindowsAudioInputDevice::get_endpoint_volume(mm_device)?,
        })
    }
    
    fn open_property_store(mm_device: *mut IMMDevice) -> Result<*mut IPropertyStore, AudioError> {
        unsafe {
            let mut props = mem::MaybeUninit::<>::uninit();
            try_com!((*mm_device).OpenPropertyStore(
                STGM_READ,
                props.as_mut_ptr()));
            Ok(props.assume_init())
        }
    }
    
    fn get_property_value(props: *mut IPropertyStore, key: REFPROPERTYKEY) -> Result<String, AudioError> {
        unsafe {
            let mut variant = mem::MaybeUninit::<>::uninit();
            try_com!((*props).GetValue(key, variant.as_mut_ptr()));
            let variant = variant.assume_init();

            Ok(OsString::from_wide(
                from_ptr(*variant.data.pwszVal())
            ).to_string_lossy().into_owned())
        }
    }
    
    fn get_endpoint_volume(mm_device: *mut IMMDevice) -> Result<*mut IAudioEndpointVolume, AudioError> {
        unsafe {
            let mut epvol = mem::MaybeUninit::<&mut IAudioEndpointVolume>::uninit();
            try_com!((*mm_device).Activate(
                &IAudioEndpointVolume::uuidof(),
                CLSCTX_ALL,
                null_mut(),
                epvol.as_mut_ptr() as *mut *mut _));
            let epvol = epvol.assume_init();
            
            Ok(epvol)
        }
    }
}

unsafe fn from_ptr<'a>(ptr: *const u16) -> &'a [u16] {
    let len = lstrlenW(ptr);
    slice::from_raw_parts(ptr, len.try_into().unwrap())
}

impl AudioInputDeviceTrait for AudioInputDevice {
    fn name(&self) -> String {
        self.name.clone()
    }
    
    fn set_mute(&self, state: bool) -> Result<bool, AudioError> {
        unsafe {
            Ok(try_com!((*self.audio_endpoint_volume).SetMute(
                BOOL::from(state), null_mut())))
        }
    }
}
