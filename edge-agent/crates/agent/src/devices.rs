// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! Device abstraction over the FFI boundary. Safe wrappers only.

use crate::config::Config;
use anyhow::Result;
use tpt_edge_ffi::{Epoch, FocuserState, MountState, WeatherSample};

pub struct DeviceHub {
    mount: tpt_edge_ffi::MountDriver,
    focuser: tpt_edge_ffi::FocuserDriver,
    weather: tpt_edge_ffi::WeatherDriver,
}

impl DeviceHub {
    pub async fn connect(_config: &Config) -> Result<Self> {
        Ok(Self {
            mount: tpt_edge_ffi::MountDriver::connect()?,
            focuser: tpt_edge_ffi::FocuserDriver::connect()?,
            weather: tpt_edge_ffi::WeatherDriver::connect()?,
        })
    }

    pub fn slew(&self, ra: f64, dec: f64, epoch: Epoch) -> Result<()> {
        self.mount.slew(ra, dec, epoch)?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.mount.stop()?;
        Ok(())
    }

    pub fn mount_state(&self) -> Result<MountState> {
        Ok(self.mount.read_encoders()?)
    }

    pub fn focus_to(&self, position: u32) -> Result<()> {
        self.focuser.move_to(position)?;
        Ok(())
    }

    pub fn focus_relative(&self, delta: i32) -> Result<()> {
        self.focuser.move_relative(delta)?;
        Ok(())
    }

    pub fn focuser_state(&self) -> Result<FocuserState> {
        Ok(self.focuser.position()?)
    }

    pub fn weather_sample(&self) -> Result<WeatherSample> {
        Ok(self.weather.sample()?)
    }
}
