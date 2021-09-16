extern crate winapi;

use std::convert::TryInto;
use std::ffi::OsString;
use std::mem;
use std::os::windows::prelude::*;
use std::ptr::null_mut;
use std::result::Result;
use std::slice;

use crate::audio_controller::{
    unknown_audio_device_error, AudioControllerTrait, AudioError, AudioInputDeviceTrait,
};

use winapi::{
    shared::{
        minwindef::BOOL,
        winerror::{S_FALSE, S_OK},
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        combaseapi::{CoCreateInstance, CLSCTX_ALL},
        coml2api::STGM_READ,
        endpointvolume::IAudioEndpointVolume,
        functiondiscoverykeys_devpkey::PKEY_Device_FriendlyName,
        mmdeviceapi::{
            eCapture, eCommunications, CLSID_MMDeviceEnumerator, IMMDevice, IMMDeviceCollection,
            IMMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        },
        objbase::CoInitialize,
        propkeydef::REFPROPERTYKEY,
        propsys::IPropertyStore,
        winbase::lstrlenW,
    },
    Interface,
};

#[macro_export]
macro_rules! EXAMPLE_PORT {
    () => {
        "COM4"
    };
}

macro_rules! try_com {
    ($expr:expr) => {
        match $expr {
            S_OK => true,
            S_FALSE => false,
            _ => {
                return Err(AudioError {
                    msg: format!("HRESULT: 0x{:X}", $expr),
                })
            }
        }
    };
}

pub struct AudioController {}

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
                device_enumerator.as_mut_ptr() as *mut *mut _
            ));
            Ok(device_enumerator.assume_init())
        }
    }

    fn get_default_communications_imm_device(
        &self,
        device_enumerator: *mut IMMDeviceEnumerator,
    ) -> Result<*mut IMMDevice, AudioError> {
        unsafe {
            let mut mm_device = mem::MaybeUninit::uninit();

            try_com!((*device_enumerator).GetDefaultAudioEndpoint(
                eCapture,
                eCommunications,
                mm_device.as_mut_ptr()
            ));
            Ok(mm_device.assume_init())
        }
    }

    fn enum_audio_endpoints(
        &self,
        device_enumerator: *mut IMMDeviceEnumerator,
    ) -> Result<*mut IMMDeviceCollection, AudioError> {
        unsafe {
            let mut p_collection = mem::MaybeUninit::<&mut IMMDeviceCollection>::uninit();

            try_com!((*device_enumerator).EnumAudioEndpoints(
                eCapture,
                DEVICE_STATE_ACTIVE,
                p_collection.as_mut_ptr() as *mut *mut _
            ));

            Ok(p_collection.assume_init())
        }
    }
}

impl AudioControllerTrait for AudioController {
    fn new() -> Self {
        unsafe {
            CoInitialize(null_mut());
        }
        AudioController {}
    }

    fn get_comms_device(&self) -> Result<Box<dyn AudioInputDeviceTrait>, AudioError> {
        let device_enumerator = self.get_device_enumerator()?;

        Ok(Box::new(AudioInputDevice::new(
            self.get_default_communications_imm_device(device_enumerator)?,
        )?))
    }

    fn get_input_device(&self, name: String) -> Result<Box<dyn AudioInputDeviceTrait>, AudioError> {
        let device_enumerator = self.get_device_enumerator()?;
        let endpoints = self.enum_audio_endpoints(device_enumerator)?;

        unsafe {
            let mut pc_devices = mem::MaybeUninit::uninit();
            try_com!((*endpoints).GetCount(pc_devices.as_mut_ptr()));
            let device_count = pc_devices.assume_init();

            let mut index = 0;
            while index < device_count {
                let mut pp_device = mem::MaybeUninit::uninit();
                try_com!((*endpoints).Item(index, pp_device.as_mut_ptr()));

                let device = AudioInputDevice::new(pp_device.assume_init())?;
                let device_name = device.name()?;

                if device_name == name {
                    return Ok(Box::new(device));
                }

                index += 1;
            }

            return Err(unknown_audio_device_error(name));
        }
    }

    fn get_input_device_names(&self) -> Result<Vec<String>, AudioError> {
        let device_enumerator = self.get_device_enumerator()?;
        let endpoints = self.enum_audio_endpoints(device_enumerator)?;
        let mut r = Vec::new();

        unsafe {
            let mut pc_devices = mem::MaybeUninit::uninit();
            try_com!((*endpoints).GetCount(pc_devices.as_mut_ptr()));
            let device_count = pc_devices.assume_init();

            let mut index = 0;
            while index < device_count {
                let mut pp_device = mem::MaybeUninit::uninit();
                try_com!((*endpoints).Item(index, pp_device.as_mut_ptr()));

                let device = AudioInputDevice::new(pp_device.assume_init())?;
                r.push(device.name()?);

                index += 1;
            }
        }
        Ok(r)
    }
}

impl AudioInputDevice {
    fn new(mm_device: *mut IMMDevice) -> Result<AudioInputDevice, AudioError> {
        // Read properties
        let props = AudioInputDevice::open_property_store(mm_device)?;

        Ok(AudioInputDevice {
            mm_device: mm_device,
            name: AudioInputDevice::get_property_value(props, &PKEY_Device_FriendlyName)?,
            audio_endpoint_volume: AudioInputDevice::get_endpoint_volume(mm_device)?,
        })
    }

    fn open_property_store(mm_device: *mut IMMDevice) -> Result<*mut IPropertyStore, AudioError> {
        unsafe {
            let mut props = mem::MaybeUninit::uninit();
            try_com!((*mm_device).OpenPropertyStore(STGM_READ, props.as_mut_ptr()));
            Ok(props.assume_init())
        }
    }

    fn get_property_value(
        props: *mut IPropertyStore,
        key: REFPROPERTYKEY,
    ) -> Result<String, AudioError> {
        unsafe {
            let mut variant = mem::MaybeUninit::uninit();
            try_com!((*props).GetValue(key, variant.as_mut_ptr()));
            let variant = variant.assume_init();

            Ok(OsString::from_wide(from_ptr(*variant.data.pwszVal()))
                .to_string_lossy()
                .into_owned())
        }
    }

    fn get_endpoint_volume(
        mm_device: *mut IMMDevice,
    ) -> Result<*mut IAudioEndpointVolume, AudioError> {
        unsafe {
            let mut epvol = mem::MaybeUninit::<&mut IAudioEndpointVolume>::uninit();
            try_com!((*mm_device).Activate(
                &IAudioEndpointVolume::uuidof(),
                CLSCTX_ALL,
                null_mut(),
                epvol.as_mut_ptr() as *mut *mut _
            ));
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
    fn name(&self) -> Result<String, AudioError> {
        Ok(self.name.clone())
    }

    fn set_mute(&self, state: bool) -> Result<bool, AudioError> {
        unsafe {
            Ok(try_com!(
                (*self.audio_endpoint_volume).SetMute(BOOL::from(state), null_mut())
            ))
        }
    }
}

impl Clone for AudioInputDevice {
    fn clone(&self) -> AudioInputDevice {
        AudioInputDevice::new(self.mm_device).unwrap()
    }
}
