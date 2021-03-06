extern crate winapi;

use std::convert::TryInto;
use std::ptr::null_mut;
use std::mem;
use std::result::Result;
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::slice;

use crate::audio_controller::{AudioController, AudioInputDevice};

use winapi::um::coml2api::STGM_READ;
use winapi::um::combaseapi::{CoCreateInstance};
use winapi::um::mmdeviceapi::{CLSID_MMDeviceEnumerator, IMMDevice, IMMDeviceEnumerator, eCapture, eCommunications};
use winapi::um::objbase::CoInitialize;
use winapi::um::propsys::IPropertyStore;
use winapi::um::functiondiscoverykeys_devpkey::PKEY_Device_FriendlyName;
use winapi::um::propkeydef::REFPROPERTYKEY;
use winapi::um::winbase::lstrlenW;
use winapi::shared::wtypesbase::CLSCTX_INPROC_SERVER;
use winapi::Interface;

macro_rules! try_com {
	($expr:expr) => (
        if $expr != 0 {
			return Err(format!("HRESULT: 0x{:X}", $expr))
		}
    )
}

pub struct WindowsAudioController {
}


pub struct WindowsAudioInputDevice {
    mm_device: *mut IMMDevice,
    name: String,
}


impl WindowsAudioController {   
    fn get_device_enumerator(&self) -> Result<*mut IMMDeviceEnumerator, String> {
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

    fn get_default_communications_imm_device(&self, device_enumerator: *mut IMMDeviceEnumerator) -> Result<*mut IMMDevice, String> {
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


impl AudioController for WindowsAudioController {
    fn new() -> Box<dyn AudioController> {
        unsafe {
            CoInitialize(null_mut());
        }
        Box::new(WindowsAudioController { })
    }
    
    fn get_comms_device(&self) -> Result<Box<dyn AudioInputDevice>, String> {
        let device_enumerator = self.get_device_enumerator().expect("IMMDeviceEnumerator");

        Ok(Box::new(WindowsAudioInputDevice::new(self.get_default_communications_imm_device(device_enumerator)?)))
    }
}

impl WindowsAudioInputDevice {
    fn new(mm_device: *mut IMMDevice) -> WindowsAudioInputDevice {
        // Read properties
        let props = WindowsAudioInputDevice::open_property_store(mm_device).expect("open");
        
        WindowsAudioInputDevice {
            mm_device: mm_device,
            name: WindowsAudioInputDevice::get_property_value(props, &PKEY_Device_FriendlyName).expect("friendly name"),
        }
    }
    
    fn open_property_store(mm_device: *mut IMMDevice) -> Result<*mut IPropertyStore, String> {
        unsafe {
            let mut props = mem::MaybeUninit::<>::uninit();
            try_com!((*mm_device).OpenPropertyStore(
                STGM_READ,
                props.as_mut_ptr()));
            Ok(props.assume_init())
        }
    }
    
    fn get_property_value(props: *mut IPropertyStore, key: REFPROPERTYKEY) -> Result<String, String> {
        unsafe {
            let mut variant = mem::MaybeUninit::<>::uninit();
            try_com!((*props).GetValue(key, variant.as_mut_ptr()));
            let variant = variant.assume_init();

            Ok(OsString::from_wide(
                from_ptr(*variant.data.pwszVal())
            ).to_string_lossy().into_owned())
        }
    }
}

unsafe fn from_ptr<'a>(ptr: *const u16) -> &'a [u16] {
    let len = lstrlenW(ptr);
    slice::from_raw_parts(ptr, len.try_into().unwrap())
}

impl AudioInputDevice for WindowsAudioInputDevice {
    fn name(&self) -> String {
        self.name.clone()
    }
    
    fn set_mute(&self, state: bool) {
        todo!()
    }
}
