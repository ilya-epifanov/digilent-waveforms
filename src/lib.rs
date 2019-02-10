extern crate time;
#[macro_use] extern crate failure_derive;

use std::ffi::CStr;
use std::mem;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::ptr;

use time::Duration;

use crate::dwf::*;
use std::fmt;
use std::fmt::Formatter;
use std::fmt::Display;

mod dwf;

pub type Result<T> = std::result::Result<T, Error>;

pub fn get_version() -> String {
    unsafe {
        let mut version = [0i8; 32];
        FDwfGetVersion(version.as_mut_ptr());

        CStr::from_ptr(mem::transmute(version.as_mut_ptr())).to_str().unwrap().to_owned()
    }
}

#[derive(PartialEq, Debug)]
pub enum ErrorKind {
    NoError = dwfercNoErc as isize,
    Unknown = dwfercUnknownError as isize,
    ApiLockTimeout = dwfercApiLockTimeout as isize,
    AlreadyOpened = dwfercAlreadyOpened as isize,
    NotSupported = dwfercNotSupported as isize,
    InvalidParameter0 = dwfercInvalidParameter0 as isize,
    InvalidParameter1 = dwfercInvalidParameter1 as isize,
    InvalidParameter2 = dwfercInvalidParameter2 as isize,
    InvalidParameter3 = dwfercInvalidParameter3 as isize,
    InvalidParameter4 = dwfercInvalidParameter4 as isize,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(match self {
            ErrorKind::NoError => "No error",
            ErrorKind::Unknown => "Unknown error",
            ErrorKind::ApiLockTimeout => "API lock timeout",
            ErrorKind::AlreadyOpened => "Device is already in use",
            ErrorKind::NotSupported => "Operation is not supported",
            ErrorKind::InvalidParameter0 => "Parameter #0 is invalid",
            ErrorKind::InvalidParameter1 => "Parameter #1 is invalid",
            ErrorKind::InvalidParameter2 => "Parameter #2 is invalid",
            ErrorKind::InvalidParameter3 => "Parameter #3 is invalid",
            ErrorKind::InvalidParameter4 => "Parameter #4 is invalid",
        })
    }
}

#[derive(Fail, Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.kind.fmt(f);
        if self.message.len() > 0 {
            f.write_str(": ");
            f.write_str(&self.message);
        }
        Ok(())
    }
}

fn get_last_error_code() -> ErrorKind {
    unsafe {
        let mut error_code: DWFERC = mem::uninitialized();
        if FDwfGetLastError((&mut error_code) as *mut DWFERC) == 0 {
            return ErrorKind::Unknown;
        }
        match error_code {
            dwfercNoErc => ErrorKind::NoError,
            dwfercUnknownError => ErrorKind::Unknown,
            dwfercApiLockTimeout => ErrorKind::ApiLockTimeout,
            dwfercAlreadyOpened => ErrorKind::AlreadyOpened,
            dwfercNotSupported => ErrorKind::NotSupported,
            dwfercInvalidParameter0 => ErrorKind::InvalidParameter0,
            dwfercInvalidParameter1 => ErrorKind::InvalidParameter1,
            dwfercInvalidParameter2 => ErrorKind::InvalidParameter2,
            dwfercInvalidParameter3 => ErrorKind::InvalidParameter3,
            dwfercInvalidParameter4 => ErrorKind::InvalidParameter4,
            _ => ErrorKind::Unknown,
        }
    }
}

fn get_last_error_message() -> String {
    unsafe {
        let mut error_message = [0i8; 512];
        FDwfGetLastErrorMsg(error_message.as_mut_ptr());

        CStr::from_ptr(mem::transmute(error_message.as_mut_ptr())).to_str().unwrap().to_owned()
    }
}

fn get_last_error() -> Error {
    Error {
        kind: get_last_error_code(),
        message: get_last_error_message(),
    }
}

#[derive(PartialEq, Debug)]
pub struct DeviceConfigInfo {
    device_ix: c_int,
    config_ix: c_int,
    pub analog_inputs: i32,
    pub analog_outputs: i32,
    pub analog_ios: i32,
    pub digital_inputs: i32,
    pub digital_outputs: i32,
    pub digital_ios: i32,
    pub analog_in_buf_size: i32,
    pub analog_out_buf_size: i32,
    pub digital_in_buf_size: i32,
    pub digital_out_buf_size: i32,
}

macro_rules! try_dwf {
    ($e: expr) => {
        {
            let ret = $e;
            if ret as BOOL == false_ as BOOL {
                return Err(get_last_error());
            }
        }
    };
}

impl DeviceConfigInfo {
    pub fn open(&self) -> Result<Device> {
        unsafe {
            let mut dev = Device {
                handle: mem::uninitialized(),
            };
            try_dwf!(FDwfDeviceConfigOpen(self.device_ix, self.config_ix, (&mut dev.handle) as *mut HDWF));
            Ok(dev)
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct DeviceInfo {
    device_ix: c_int,
    pub id: i32,
    pub revision: i32,
    pub user_name: String,
    pub name: String,
    pub serial: String,
    pub in_use: bool,
    pub configs: Vec<DeviceConfigInfo>,
}

#[derive(PartialEq, Debug)]
pub struct DeviceInfoList {
    pub devices: Vec<DeviceInfo>,
}

pub fn devices() -> Result<DeviceInfoList> {
    unsafe {
        let mut devices_cnt: c_int = 0;
        try_dwf!(FDwfEnum(enumfilterAll, &mut devices_cnt as *mut c_int));
        let mut devices = Vec::with_capacity(devices_cnt as usize);

        for device_ix in 0..devices_cnt {
            let mut id: DEVID = mem::uninitialized();
            let mut ver: DEVVER = mem::uninitialized();

            try_dwf!(FDwfEnumDeviceType(device_ix, &mut id as *mut DEVID, &mut ver as *mut DEVVER));

            let mut in_use: BOOL = mem::uninitialized();
            try_dwf!(FDwfEnumDeviceIsOpened(device_ix, &mut in_use as *mut BOOL));

            let mut user_name = [0 as c_char; 32];
            try_dwf!(FDwfEnumUserName(device_ix, user_name.as_mut_ptr()));

            let mut name = [0 as c_char; 32];
            try_dwf!(FDwfEnumDeviceName(device_ix, name.as_mut_ptr()));

            let mut serial = [0 as c_char; 32];
            try_dwf!(FDwfEnumSN(device_ix, serial.as_mut_ptr()));

            let mut configs_cnt: c_int = 0;
            try_dwf!(FDwfEnumConfig(device_ix, &mut configs_cnt as *mut c_int));

            let mut configs = Vec::with_capacity(configs_cnt as usize);

            for config_ix in 0..configs_cnt {
                let mut analog_inputs: c_int = 0;
                try_dwf!(FDwfEnumConfigInfo(config_ix, DECIAnalogInChannelCount, &mut analog_inputs as *mut c_int));

                let mut analog_outputs: c_int = 0;
                try_dwf!(FDwfEnumConfigInfo(config_ix, DECIAnalogInChannelCount, &mut analog_outputs as *mut c_int));

                let mut analog_ios: c_int = 0;
                try_dwf!(FDwfEnumConfigInfo(config_ix, DECIAnalogInChannelCount, &mut analog_ios as *mut c_int));

                let mut digital_inputs: c_int = 0;
                try_dwf!(FDwfEnumConfigInfo(config_ix, DECIDigitalInChannelCount, &mut digital_inputs as *mut c_int));

                let mut digital_outputs: c_int = 0;
                try_dwf!(FDwfEnumConfigInfo(config_ix, DECIDigitalInChannelCount, &mut digital_outputs as *mut c_int));

                let mut digital_ios: c_int = 0;
                try_dwf!(FDwfEnumConfigInfo(config_ix, DECIDigitalInChannelCount, &mut digital_ios as *mut c_int));

                let mut analog_in_buf_size: c_int = 0;
                try_dwf!(FDwfEnumConfigInfo(config_ix, DECIAnalogInBufferSize, &mut analog_in_buf_size as *mut c_int));

                let mut analog_out_buf_size: c_int = 0;
                try_dwf!(FDwfEnumConfigInfo(config_ix, DECIAnalogOutBufferSize, &mut analog_out_buf_size as *mut c_int));

                let mut digital_in_buf_size: c_int = 0;
                try_dwf!(FDwfEnumConfigInfo(config_ix, DECIDigitalInBufferSize, &mut digital_in_buf_size as *mut c_int));

                let mut digital_out_buf_size: c_int = 0;
                try_dwf!(FDwfEnumConfigInfo(config_ix, DECIDigitalOutBufferSize, &mut digital_out_buf_size as *mut c_int));

                configs.insert(config_ix as usize, DeviceConfigInfo {
                    device_ix,
                    config_ix,
                    analog_inputs,
                    analog_outputs,
                    analog_ios,
                    digital_inputs,
                    digital_outputs,
                    digital_ios,
                    analog_in_buf_size,
                    analog_out_buf_size,
                    digital_in_buf_size,
                    digital_out_buf_size,
                })
            }

            devices.insert(device_ix as usize, DeviceInfo {
                device_ix,
                id,
                revision: ver,
                user_name: CStr::from_ptr(mem::transmute(user_name.as_mut_ptr())).to_str().unwrap().to_owned(),
                name: CStr::from_ptr(mem::transmute(name.as_mut_ptr())).to_str().unwrap().to_owned(),
                serial: CStr::from_ptr(mem::transmute(serial.as_mut_ptr())).to_str().unwrap().to_owned(),
                in_use: in_use != 0,
                configs,
            })
        }

        Ok(DeviceInfoList { devices })
    }
}

pub struct AnalogOutNode<'a> {
    out: &'a AnalogOut<'a>,
    ix: c_int,
}

impl<'a> AnalogOutNode<'a> {
    pub fn set_function(&self, func: AnalogOutFunction) -> Result<()> {
        unsafe {
            match func {
                AnalogOutFunction::Triangle { frequency, amplitude, offset, symmetry, phase_deg } => {
                    try_dwf!(FDwfAnalogOutNodeFunctionSet(self.out.device.handle, self.out.ix, self.ix, funcTriangle));
                    try_dwf!(FDwfAnalogOutNodeFrequencySet(self.out.device.handle, self.out.ix, self.ix, frequency));
                    try_dwf!(FDwfAnalogOutNodeAmplitudeSet(self.out.device.handle, self.out.ix, self.ix, amplitude));
                    try_dwf!(FDwfAnalogOutNodeOffsetSet(self.out.device.handle, self.out.ix, self.ix, offset));
                    try_dwf!(FDwfAnalogOutNodeSymmetrySet(self.out.device.handle, self.out.ix, self.ix, symmetry));
                    try_dwf!(FDwfAnalogOutNodePhaseSet(self.out.device.handle, self.out.ix, self.ix, phase_deg));
                }
            }
        }
        Ok(())
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        unsafe {
            try_dwf!(FDwfAnalogOutNodeEnableSet(self.out.device.handle, self.out.ix, self.ix, to_c_bool(enabled)));
        }
        Ok(())
    }
}

pub enum AnalogOutFunction {
    Triangle { frequency: f64, amplitude: f64, offset: f64, symmetry: f64, phase_deg: f64 }
}

pub struct AnalogOut<'a> {
    device: &'a Device,
    ix: c_int,
}

impl<'a> AnalogOut<'a> {
    pub fn node(&self, ix: u32) -> AnalogOutNode {
        AnalogOutNode {
            out: &self,
            ix: ix as c_int,
        }
    }

    pub fn set_duration(&self, duration: Duration) -> Result<()> {
        unsafe {
            try_dwf!(FDwfAnalogOutRunSet(self.device.handle, self.ix, duration.num_nanoseconds().unwrap() as f64 / 1e9));
        }
        Ok(())
    }

    pub fn set_repeat_count(&self, repeat_cnt: i32) -> Result<()> {
        unsafe {
            try_dwf!(FDwfAnalogOutRepeatSet(self.device.handle, self.ix, repeat_cnt));
        }
        Ok(())
    }

    pub fn start(&self) -> Result<()> {
        unsafe {
            try_dwf!(FDwfAnalogOutConfigure(self.device.handle, self.ix, to_c_bool(true)));
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        unsafe {
            try_dwf!(FDwfAnalogOutConfigure(self.device.handle, self.ix, to_c_bool(false)));
        }
        Ok(())
    }
}

pub struct AnalogIO<'a> {
    device: &'a Device,
}

impl<'a> AnalogIO<'a> {
    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        unsafe {
            try_dwf!(FDwfAnalogIOEnableSet(self.device.handle, to_c_bool(enabled)));
        }
        Ok(())
    }

    pub fn channel(&self, ix: i32) -> AnalogIOChannel {
        AnalogIOChannel {
            io: self,
            ix: ix as c_int,
        }
    }
}

pub struct AnalogIOChannel<'a> {
    io: &'a AnalogIO<'a>,
    ix: c_int,
}

impl<'a> AnalogIOChannel<'a> {
    pub fn node(&self, ix: i32) -> AnalogIOChannelNode {
        AnalogIOChannelNode {
            channel: self,
            ix: ix as c_int,
        }
    }
}

pub struct AnalogIOChannelNode<'a> {
    channel: &'a AnalogIOChannel<'a>,
    ix: c_int,
}

impl<'a> AnalogIOChannelNode<'a> {
    pub fn set_value(&self, value: f64) -> Result<()> {
        unsafe {
            try_dwf!(FDwfAnalogIOChannelNodeSet(self.channel.io.device.handle, self.channel.ix, self.ix, value));
        }
        Ok(())
    }
}


pub struct AnalogIn<'a> {
    device: &'a Device,
}

impl<'a> AnalogIn<'a> {
    pub fn start(&self) -> Result<()> {
        unsafe {
            try_dwf!(FDwfAnalogInConfigure(self.device.handle, to_c_bool(false), to_c_bool(true)));
        }
        Ok(())
    }

    pub fn set_frequency(&self, freq: f64) -> Result<()> {
        unsafe {
            try_dwf!(FDwfAnalogInFrequencySet(self.device.handle, freq));
        }
        Ok(())
    }

    pub fn set_buffer_size(&self, buf_size: u32) -> Result<()> {
        unsafe {
            try_dwf!(FDwfAnalogInBufferSizeSet(self.device.handle, buf_size as i32));
        }
        Ok(())
    }

    pub fn set_record_mode(&self, length: f64) -> Result<()> {
        unsafe {
            try_dwf!(FDwfAnalogInRecordLengthSet(self.device.handle, length));
            try_dwf!(FDwfAnalogInAcquisitionModeSet(self.device.handle, acqmodeRecord as ACQMODE));
        }
        Ok(())
    }

    pub fn channel(&self, ix: i32) -> AnalogInChannel {
        AnalogInChannel {
            input: &self,
            ix: ix as c_int,
        }
    }

    pub fn get_status(&self) -> Result<AnalogAcquisitionStatus> {
        unsafe {
            let mut state: DwfState = mem::uninitialized();
            try_dwf!(FDwfAnalogInStatus(self.device.handle, to_c_bool(true), (&mut state) as *mut DwfState));
            Ok(match state {
                DwfStateReady => AnalogAcquisitionStatus::Ready,
                DwfStateConfig => AnalogAcquisitionStatus::Config,
                DwfStatePrefill => AnalogAcquisitionStatus::Prefill,
                DwfStateArmed => AnalogAcquisitionStatus::Armed,
                DwfStateWait => AnalogAcquisitionStatus::Waiting,
                DwfStateRunning => AnalogAcquisitionStatus::Running,
                DwfStateDone => AnalogAcquisitionStatus::Done,
                _ => panic!(),
            })
        }
    }

    pub fn get_samples_left(&self) -> Result<i32> {
        unsafe {
            let mut ret = mem::uninitialized();
            try_dwf!(FDwfAnalogInStatusSamplesLeft(self.device.handle, &mut ret as *mut c_int));
            Ok(ret)
        }
    }

    pub fn get_record_status(&self) -> Result<(i32, i32, i32)> {
        unsafe {
            let (mut available, mut lost, mut corrupted) = mem::uninitialized();
            try_dwf!(FDwfAnalogInStatusRecord(self.device.handle,
                &mut available as *mut c_int,
                &mut lost as *mut c_int,
                &mut corrupted as *mut c_int
            ));
            Ok((available, lost, corrupted))
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum AnalogAcquisitionStatus {
    Ready = DwfStateReady as isize,
    Config = DwfStateConfig as isize,
    Prefill = DwfStatePrefill as isize,
    Armed = DwfStateArmed as isize,
    Waiting = DwfStateWait as isize,
    Running = DwfStateRunning as isize,
    Done = DwfStateDone as isize,
}

pub struct AnalogInChannel<'a> {
    input: &'a AnalogIn<'a>,
    ix: c_int,
}

impl<'a> AnalogInChannel<'a> {
    pub fn set_offset(&self, offset: f64) -> Result<()> {
        unsafe {
            try_dwf!(FDwfAnalogInChannelOffsetSet(self.input.device.handle, self.ix, offset));
        }
        Ok(())
    }

    pub fn set_range(&self, range: f64) -> Result<()> {
        unsafe {
            try_dwf!(FDwfAnalogInChannelRangeSet(self.input.device.handle, self.ix, range));
        }
        Ok(())
    }

    pub fn fetch_samples(&self, dest: &mut Vec<f64>, available: i32) -> Result<()> {
        unsafe {
            let original_len = dest.len();
            dest.reserve(available as usize);
            dest.set_len(original_len + available as usize);
            try_dwf!(FDwfAnalogInStatusData(self.input.device.handle, self.ix,
                dest.as_mut_ptr().offset(original_len as isize), available));
        }
        Ok(())
    }
}


pub struct Device {
    handle: HDWF,
}

impl Device {
    pub fn set_auto_configure(&self, enabled: bool) -> Result<()> {
        unsafe {
            try_dwf!(FDwfDeviceAutoConfigureSet(self.handle, to_c_bool(enabled)));
        }
        Ok(())
    }

    pub fn reset(&self) -> Result<()> {
        unsafe {
            try_dwf!(FDwfDeviceReset(self.handle));
        }
        Ok(())
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        unsafe {
            try_dwf!(FDwfDeviceEnableSet(self.handle, to_c_bool(enabled)));
        }
        Ok(())
    }

    pub fn analog_out(&self, ix: u32) -> AnalogOut {
        AnalogOut {
            device: &self,
            ix: ix as c_int,
        }
    }

    pub fn analog_io(&self) -> AnalogIO {
        AnalogIO {
            device: &self,
        }
    }

    pub fn analog_input(&self) -> AnalogIn {
        AnalogIn {
            device: &self,
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            FDwfDeviceClose(self.handle);
        }
    }
}
