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
use winapi::um::propidl::PROPVARIANT;
use winapi::um::propsys::IPropertyStore;
use winapi::um::functiondiscoverykeys_devpkey::PKEY_Device_FriendlyName;
use winapi::um::propkeydef::REFPROPERTYKEY;
use winapi::um::winbase::lstrlenW;
use winapi::shared::ntdef::HRESULT;
use winapi::shared::wtypesbase::CLSCTX_INPROC_SERVER;
use winapi::Interface;

// https://gist.github.com/DrMetallius/084115493deb21148a6bef9326b47ea6
macro_rules! try_com {
	($expr:expr) => (if $expr != 0 {
			return Err(format!("HRESULT: 0x{:X}", $expr))
		})
}

macro_rules! new_ref {
//	($param:ident, $type:ty) => {let mut $param: &mut $type = mem::uninitialized(); }
    ($param:ident, $type:ty) => {let mut $param: *mut $type = mem::zeroed(); }
//	($param:ident, $type:ty) => {let mut $param = mem::MaybeUninit::<*mut $type >::uninit(); }
}

macro_rules! out_param {
//    ($param:ident) => {&mut $param as *mut _ as *mut _}
//    ($param:ident) => {&mut $param.as_mut_ptr() as *mut _ as *mut _}
//    ($param:ident) => {&mut $param.as_mut_ptr() as *mut *mut _ as *mut *mut _}
    ($param:ident) => {&mut $param as *mut *mut _ as *mut *mut _}
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
            new_ref!(device_enumerator, IMMDeviceEnumerator);
            try_com!(CoCreateInstance(
                &CLSID_MMDeviceEnumerator,
                null_mut(),
                CLSCTX_INPROC_SERVER,
                &IMMDeviceEnumerator::uuidof() as *const _,
                out_param!(device_enumerator)));
//            Ok(device_enumerator.assume_init())
            Ok(device_enumerator)
        }
    }
    
    /*
    fn enum_audio_endpoints(&self, device_enumerator: *mut IMMDeviceEnumerator) -> Result<*mut IMMDeviceCollection, String> {
        unsafe {
            new_ref!(device_collection, IMMDeviceCollection);
            try_com!((*device_enumerator).EnumAudioEndpoints(
                eRender,
                DEVICE_STATE_ACTIVE,
                out_param!(device_collection)));
//            Ok(device_collection.assume_init())
            Ok(device_collection)
        }
    }
    */
    
    fn get_default_communications_imm_device(&self, device_enumerator: *mut IMMDeviceEnumerator) -> Result<*mut IMMDevice, String> {
        unsafe {
            new_ref!(mm_device, IMMDevice);
            try_com!((*device_enumerator).GetDefaultAudioEndpoint(
                eCapture,
                eCommunications,
                out_param!(mm_device)));
            Ok(mm_device)
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
            new_ref!(props, IPropertyStore);
            try_com!((*mm_device).OpenPropertyStore(
                STGM_READ,
                &mut props));
            Ok(props)
        }
    }
    
    fn get_property_value(props: *mut IPropertyStore, key: REFPROPERTYKEY) -> Result<String, String> {
        unsafe {
            let mut variant = mem::uninitialized();
            // new_ref!(var_name, PROPVARIANT);
            try_com!((*props).GetValue(key, &mut variant));
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
