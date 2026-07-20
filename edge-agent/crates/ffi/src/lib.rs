// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! Safe FFI boundary for INDI / ASCOM device drivers.
//! See docs/ffi-boundary.md for the safety contract.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("driver not connected")]
    NotConnected,
    #[error("invalid argument: {0}")]
    InvalidArg(String),
    #[error("driver returned error code {0}")]
    Driver(i32),
    #[error("null pointer from driver")]
    NullPtr,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Epoch {
    J2000,
    JNow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountState {
    pub ra: f64,
    pub dec: f64,
    pub alt: f64,
    pub az: f64,
    pub tracking: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocuserState {
    pub position: u32,
    pub temperature: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherSample {
    pub temp: f64,
    pub humidity: f64,
    pub pressure: f64,
    pub wind_speed: f64,
    pub dew_point: f64,
    pub cloud_cover: f64,
}

// ----- Safe driver wrappers (unsafe layer confined to `sys` module) -----

pub struct MountDriver(*mut sys::indi_handle_t);
pub struct FocuserDriver(*mut sys::indi_handle_t);
pub struct WeatherDriver(*mut sys::indi_handle_t);

impl MountDriver {
    pub fn connect() -> Result<Self, DeviceError> {
        let h = sys::connect(b"mount\0".as_ptr() as *const _)?;
        Ok(MountDriver(h))
    }
    pub fn slew(&self, ra: f64, dec: f64, _epoch: Epoch) -> Result<(), DeviceError> {
        sys::mount_slew(self.0, ra, dec)
    }
    pub fn stop(&self) -> Result<(), DeviceError> {
        sys::mount_stop(self.0)
    }
    pub fn read_encoders(&self) -> Result<MountState, DeviceError> {
        sys::mount_read(self.0)
    }
}

impl FocuserDriver {
    pub fn connect() -> Result<Self, DeviceError> {
        let h = sys::connect(b"focuser\0".as_ptr() as *const _)?;
        Ok(FocuserDriver(h))
    }
    pub fn move_to(&self, position: u32) -> Result<(), DeviceError> {
        sys::focuser_move(self.0, position as i64)
    }
    pub fn move_relative(&self, delta: i32) -> Result<(), DeviceError> {
        sys::focuser_move(self.0, delta as i64)
    }
    pub fn position(&self) -> Result<FocuserState, DeviceError> {
        sys::focuser_read(self.0)
    }
}

impl WeatherDriver {
    pub fn connect() -> Result<Self, DeviceError> {
        let h = sys::connect(b"weather\0".as_ptr() as *const _)?;
        Ok(WeatherDriver(h))
    }
    pub fn sample(&self) -> Result<WeatherSample, DeviceError> {
        sys::weather_sample(self.0)
    }
}

impl Drop for MountDriver {
    fn drop(&mut self) {
        let _ = sys::disconnect(self.0);
    }
}
impl Drop for FocuserDriver {
    fn drop(&mut self) {
        let _ = sys::disconnect(self.0);
    }
}
impl Drop for WeatherDriver {
    fn drop(&mut self) {
        let _ = sys::disconnect(self.0);
    }
}

/// Low-level `unsafe` FFI declarations. Confined to this module.
mod sys {
    use super::{DeviceError, FocuserState, MountState, WeatherSample};

    #[repr(C)]
    pub struct indi_handle_t {
        _private: [u8; 0],
    }

    #[repr(C)]
    struct WeatherSampleRaw {
        temp: f64,
        humidity: f64,
        pressure: f64,
        wind_speed: f64,
        dew_point: f64,
        cloud_cover: f64,
    }

    extern "C" {
        fn indi_connect(device: *const u8) -> *mut indi_handle_t;
        fn indi_disconnect(handle: *const indi_handle_t) -> i32;
        fn indi_mount_slew(handle: *const indi_handle_t, ra: f64, dec: f64) -> i32;
        fn indi_mount_stop(handle: *const indi_handle_t) -> i32;
        fn indi_mount_read(handle: *const indi_handle_t, ra: *mut f64, dec: *mut f64, status: *mut i32) -> i32;
        fn indi_focuser_move(handle: *const indi_handle_t, steps: i64) -> i32;
        fn indi_focuser_read(handle: *const indi_handle_t, pos: *mut u32, temp: *mut f64) -> i32;
        fn indi_weather_sample(handle: *const indi_handle_t, out: *mut WeatherSampleRaw) -> i32;
    }

    // Safe wrappers around the extern declarations above.
    pub fn connect(device: *const u8) -> Result<*mut indi_handle_t, DeviceError> {
        unsafe {
            let p = indi_connect(device);
            if p.is_null() {
                Err(DeviceError::NullPtr)
            } else {
                Ok(p)
            }
        }
    }

    pub fn disconnect(handle: *const indi_handle_t) -> Result<(), DeviceError> {
        unsafe {
            let rc = indi_disconnect(handle);
            if rc == 0 { Ok(()) } else { Err(DeviceError::Driver(rc)) }
        }
    }

    pub fn mount_slew(handle: *const indi_handle_t, ra: f64, dec: f64) -> Result<(), DeviceError> {
        unsafe {
            let rc = indi_mount_slew(handle, ra, dec);
            if rc == 0 { Ok(()) } else { Err(DeviceError::Driver(rc)) }
        }
    }

    pub fn mount_stop(handle: *const indi_handle_t) -> Result<(), DeviceError> {
        unsafe {
            let rc = indi_mount_stop(handle);
            if rc == 0 { Ok(()) } else { Err(DeviceError::Driver(rc)) }
        }
    }

    pub fn mount_read(handle: *const indi_handle_t) -> Result<MountState, DeviceError> {
        unsafe {
            let mut ra = 0f64;
            let mut dec = 0f64;
            let mut status = 0i32;
            let rc = indi_mount_read(handle, &mut ra, &mut dec, &mut status);
            if rc == 0 {
                Ok(MountState {
                    ra,
                    dec,
                    alt: 0.0,
                    az: 0.0,
                    tracking: status == 1,
                    status: if status == 1 { "tracking" } else { "idle" }.into(),
                })
            } else {
                Err(DeviceError::Driver(rc))
            }
        }
    }

    pub fn focuser_move(handle: *const indi_handle_t, steps: i64) -> Result<(), DeviceError> {
        unsafe {
            let rc = indi_focuser_move(handle, steps);
            if rc == 0 { Ok(()) } else { Err(DeviceError::Driver(rc)) }
        }
    }

    pub fn focuser_read(handle: *const indi_handle_t) -> Result<FocuserState, DeviceError> {
        unsafe {
            let mut pos = 0u32;
            let mut temp = 0f64;
            let rc = indi_focuser_read(handle, &mut pos, &mut temp);
            if rc == 0 {
                Ok(FocuserState { position: pos, temperature: temp })
            } else {
                Err(DeviceError::Driver(rc))
            }
        }
    }

    pub fn weather_sample(handle: *const indi_handle_t) -> Result<WeatherSample, DeviceError> {
        unsafe {
            let mut raw = WeatherSampleRaw {
                temp: 0.0,
                humidity: 0.0,
                pressure: 0.0,
                wind_speed: 0.0,
                dew_point: 0.0,
                cloud_cover: 0.0,
            };
            let rc = indi_weather_sample(handle, &mut raw);
            if rc == 0 {
                Ok(WeatherSample {
                    temp: raw.temp,
                    humidity: raw.humidity,
                    pressure: raw.pressure,
                    wind_speed: raw.wind_speed,
                    dew_point: raw.dew_point,
                    cloud_cover: raw.cloud_cover,
                })
            } else {
                Err(DeviceError::Driver(rc))
            }
        }
    }
}

