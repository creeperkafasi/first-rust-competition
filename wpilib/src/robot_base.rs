/* PORTIONS OF THIS FILE WERE ORIGINALLY DISTRIBUTED WITH THE FOLLOWING LICENSE

"""
MIT License
Copyright (c) 2017 Rust for Robotics Developers
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"""

Copyright 2018 First Rust Competition Developers.
Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
*/

use super::{ds::*, observe};
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use wpilib_sys::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RobotBaseInitError {
    HalInitFailed,
    AlreadyInit,
}

// Should we give in and use lazy static?
static ROBOT_INITED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct RobotBase {}

/// Though many methods are static for convenience, they will error if a
/// a `RobotBase` has not already been constructed.
impl RobotBase {
    /// Create a new robot, initializing hardware in the process.
    /// Call before initializing any other wpilib stuff.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Result<RobotBase, RobotBaseInitError> {
        if ROBOT_INITED.compare_and_swap(false, true, Ordering::AcqRel) {
            return Err(RobotBaseInitError::AlreadyInit);
        }

        if unsafe { HAL_Initialize(500, 0) } == 0 {
            return Err(RobotBaseInitError::HalInitFailed);
        }

        usage::report(usage::resource_types::Language, unsafe {
            mem::transmute(*b"Rust")
        });
        println!("\n********** Hardware Init **********\n");
        Ok(RobotBase {})
    }

    /// Call when your robot is ready to be enabled.
    /// Make sure your hardware and threads have been created, etc.
    pub fn start_competition() {
        observe::start();
        println!("\n********** Robot program starting **********\n");
    }

    pub fn make_ds(&self) -> DriverStation {
        DriverStation::from_base(self).expect("HAL FAILED ON DS CREATION")
    }

    /// Return the FPGA Version number.
    ///
    /// For now, expect this to be competition year.
    #[inline(always)]
    pub fn fpga_version() -> HalResult<i32> {
        hal_call!(HAL_GetFPGAVersion())
    }

    /// Return the FPGA Revision number.
    ///
    /// The format of the revision is 3 numbers. The 12 most significant bits are the
    /// Major Revision. The next 8 bits are the Minor Revision. The 12 least
    /// significant bits are the Build Number.
    #[inline(always)]
    pub fn fpga_revision() -> HalResult<i64> {
        hal_call!(HAL_GetFPGARevision())
    }

    /// Read the microsecond-resolution timer on the FPGA.
    ///
    /// Returns The current time in microseconds according to the FPGA (since FPGA
    ///         reset).
    #[inline(always)]
    pub fn fpga_time() -> HalResult<u64> {
        hal_call!(HAL_GetFPGATime())
    }

    /// Read the microsecond-resolution timer
    /// on the FPGA as a `std::time::Duration`.
    pub fn fpga_time_duration() -> HalResult<Duration> {
        let usec = Self::fpga_time()?;
        let sec: u64 = usec / 1_000_000;
        let nsec: u32 = (usec % 1_000_000) as u32 * 1000;
        Ok(Duration::new(sec, nsec))
    }
    /// Get the state of the "USER" button on the roboRIO.
    ///
    /// True if the button is currently pressed.
    #[inline(always)]
    pub fn user_button() -> HalResult<bool> {
        Ok(hal_call!(HAL_GetFPGAButton())? != 0)
    }

    /// Reuturns true if the robot is browned out.
    pub fn is_browned_out() -> HalResult<bool> {
        Ok(hal_call!(HAL_GetBrownedOut())? != 0)
    }

    /// Returns true if outputs are enabled.
    /// A false result could be caused by a disabled robot or a brownout.
    pub fn is_system_active() -> HalResult<bool> {
        Ok(hal_call!(HAL_GetSystemActive())? != 0)
    }

    /// Get the robot's current battery voltage.
    pub fn battery_voltage() -> HalResult<f64> {
        hal_call!(HAL_GetVinVoltage())
    }
}

impl Drop for RobotBase {
    fn drop(&mut self) {
        // In WPILibc, this is handled by the destructor of a static variable,
        // the one ds instance, which runs after main() finishes. Rust does not
        // execute anything after main, so this is the closest approximation.
        unsafe {
            HAL_ReleaseDSMutex();
        }
    }
}
