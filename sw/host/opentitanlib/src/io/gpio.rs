// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};
use structopt::clap::arg_enum;
use thiserror::Error;

use crate::impl_serializable_error;
use crate::transport::TransportError;

/// Errors related to the GPIO interface.
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum GpioError {
    #[error("Invalid pin name {0}")]
    InvalidPinName(String),
    #[error("Invalid pin number {0}")]
    InvalidPinNumber(u8),
    /// The current mode of the pin (input) does not support the requested operation (set
    /// level).
    #[error("Invalid mode for pin {0}")]
    InvalidPinMode(u8),
    /// The hardware does not support the requested mode (open drain, pull down input, etc.)
    #[error("Unsupported mode {0} requested")]
    UnsupportedPinMode(PinMode),
    /// The hardware does not support the requested mode (open drain, pull down input, etc.)
    #[error("Unsupported pull mode {0} requested")]
    UnsupportedPullMode(PullMode),
    #[error("Conflicting pin configurations for pin {0}: host:{1}, target:{2}")]
    PinModeConflict(String, String, String),
    #[error("Conflicting pin logic values for pin {0}: host:{1}, target:{2}")]
    PinValueConflict(String, String, String),
    #[error("Undefined pin logic value for pin {0}")]
    PinValueUndefined(String),
    #[error("Unsupported voltage {0}V requested")]
    UnsupportedPinVoltage(f32),
    #[error("Generic error: {0}")]
    Generic(String),
}
impl_serializable_error!(GpioError);

arg_enum! {
    /// Mode of I/O pins.
    #[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub enum PinMode {
        Input,
        PushPull,
        OpenDrain,
        AnalogInput,
        AnalogOutput,
        Alternate, // Pin used for UART/SPI/I2C or something else
    }
}

arg_enum! {
    /// Mode of weak pull (relevant in Input and OpenDrain modes).
    #[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub enum PullMode {
        None,
        PullUp,
        PullDown,
    }
}

/// A trait which represents a single GPIO pin.
pub trait GpioPin {
    /// Reads the value of the the GPIO pin.
    fn read(&self) -> Result<bool>;

    /// Sets the value of the GPIO pin to `value`.
    fn write(&self, value: bool) -> Result<()>;

    /// Sets the mode of the GPIO pin as input, output, or open drain I/O.
    fn set_mode(&self, mode: PinMode) -> Result<()>;

    /// Sets the weak pull resistors of the GPIO pin.
    fn set_pull_mode(&self, mode: PullMode) -> Result<()>;

    /// Reads the analog value of the the GPIO pin in Volts. `AnalogInput` mode disables digital
    /// circuitry for better results, but this method may also work in other modes.
    fn analog_read(&self) -> Result<f32> {
        Err(TransportError::UnsupportedOperation.into())
    }

    /// Sets the analog value of the GPIO pin to `value` Volts, must be in `AnalogOutput` mode.
    fn analog_write(&self, _volts: f32) -> Result<()> {
        Err(TransportError::UnsupportedOperation.into())
    }

    /// Simultaneously sets mode, value, and weak pull, some transports may guarantee atomicity.
    fn set(
        &self,
        mode: Option<PinMode>,
        value: Option<bool>,
        pull: Option<PullMode>,
        analog_value: Option<f32>,
    ) -> Result<()> {
        // Transports must override this function for truly atomic behavior.  Default
        // implementation below applies each setting separately.
        if let Some(mode) = mode {
            self.set_mode(mode)?;
        }
        if let Some(pull) = pull {
            self.set_pull_mode(pull)?;
        }
        if let Some(value) = value {
            self.write(value)?;
        }
        if let Some(analog_value) = analog_value {
            self.analog_write(analog_value)?;
        }
        Ok(())
    }

    /// Not meant for API clients, this method returns the pin name as it is known to the
    /// transport (which may have been through one or more alias mappings from the name provided
    /// by the API client.)  This method is used by implementations of `GpioMonitoring`.
    fn get_internal_pin_name(&self) -> Option<&str> {
        None
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Edge {
    Rising,
    Falling,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClockNature {
    /// Unix time can be computed as (t + offset) / resolution, where t is a 64-bit timestamp
    /// value from `MonitoringEvent`.
    Wallclock {
        /// If resolution is microseconds, `resolution` will be 1_000_000.
        resolution: u64,
        /// Offset relative to Unix epoch, measured according to above resolution.
        offset: Option<u64>,
    },
    /// The 64-bit timestamp values could be emulator clock counts, or some other measure that
    /// increases monotonically, but not necessarily uniformly in relation to wall clock time.
    Unspecified,
}

/// Represents an edge detected on the GPIO pin.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MonitoringEvent {
    /// Identification of the signal that had an event, in the form of an index into the array
    /// originally passed to `monitoring_read()`.
    pub signal_index: u8,
    /// Rising or falling edge
    pub edge: Edge,
    /// Timestamp of the edge, resolution and epoch is transport-specific, more information in
    /// `ClockNature`.
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitoringStartResponse {
    /// Transport timestamp at the time monitoring started.
    pub timestamp: u64,
    /// Initial logic level for each of the given pins.
    pub initial_levels: Vec<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitoringReadResponse {
    /// List of events having occurred since the start or the last read.
    pub events: Vec<MonitoringEvent>,
    /// All events at or before this timestamp are guaranteed to be included.
    pub timestamp: u64,
}

/// A trait implemented by transports which support advanced edge-detection on GPIO pins.  This
/// trait allows monitoring a set of pins, and getting a stream of "events" (rising and falling
/// edges with timestamps) for any change among the set.
pub trait GpioMonitoring {
    fn get_clock_nature(&self) -> Result<ClockNature>;

    /// Set up edge trigger detection on the given set of pins, transport will buffer the list
    /// internally, return the initial level of each of the given pins.
    fn monitoring_start(&self, pins: &[&dyn GpioPin]) -> Result<MonitoringStartResponse>;

    /// Retrieve list of events detected thus far, optionally stopping the possibly expensive edge
    /// detection.  Buffer overrun will be reported as an `Err`, and result in the stopping of the
    /// edge detection irrespective of the parameter value.
    fn monitoring_read(
        &self,
        pins: &[&dyn GpioPin],
        continue_monitoring: bool,
    ) -> Result<MonitoringReadResponse>;
}
