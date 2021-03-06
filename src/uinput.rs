use InputEvent;
use libc::c_int;
use device::Device;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use nix::errno::Errno;

use util::*;

/// Opaque struct representing an evdev uinput device
pub struct UInputDevice {
    raw: *mut raw::libevdev_uinput
}

impl UInputDevice {
    /// Create a uinput device based on the given libevdev device.
    ///
    /// The uinput device will be an exact copy of the libevdev device, minus
    /// the bits that uinput doesn't allow to be set.
    pub fn create_from_device(device: &Device) -> Result<UInputDevice, Errno> {
        let mut libevdev_uinput = 0 as *mut _;
        let result = unsafe {
            raw::libevdev_uinput_create_from_device(device.raw, raw::LIBEVDEV_UINPUT_OPEN_MANAGED, &mut libevdev_uinput)
        };

        match result {
            0 => Ok(UInputDevice { raw: libevdev_uinput }),
            error => Err(Errno::from_i32(-error))
        }
    }

    /// Return the device node representing this uinput device.
    ///
    /// This relies on libevdev_uinput_get_syspath() to provide a valid syspath.
    string_getter!(devnode, libevdev_uinput_get_devnode);

    /// Return the syspath representing this uinput device.
    ///
    /// If the UI_GET_SYSNAME ioctl not available, libevdev makes an educated
    /// guess. The UI_GET_SYSNAME ioctl is available since Linux 3.15.
    ///
    /// The syspath returned is the one of the input node itself
    /// (e.g. /sys/devices/virtual/input/input123), not the syspath of the
    /// device node returned with libevdev_uinput_get_devnode().
    string_getter!(syspath, libevdev_uinput_get_syspath);

    /// Return the file descriptor used to create this uinput device.
    ///
    /// This is the fd pointing to /dev/uinput. This file descriptor may be used
    /// to write events that are emitted by the uinput device. Closing this file
    ///  descriptor will destroy the uinput device.
    pub fn fd(&self) -> Option<File> {
        let result = unsafe {
            raw::libevdev_uinput_get_fd(self.raw)
        };

        if result == 0 {
            None
        } else {
            unsafe {
                let f = File::from_raw_fd(result);
                Some(f)
            }
        }
    }

    /// Post an event through the uinput device.
    ///
    /// It is the caller's responsibility that any event sequence is terminated
    /// with an EV_SYN/SYN_REPORT/0 event. Otherwise, listeners on the device
    /// node will not see the events until the next EV_SYN event is posted.
    pub fn write_event(&self, event: &InputEvent) -> Result<(), Errno> {
        let (ev_type, ev_code) = event_code_to_int(&event.event_code);
        let ev_value = event.value as c_int;

        let result = unsafe {
            raw::libevdev_uinput_write_event(self.raw, ev_type, ev_code, ev_value)
        };

        match result {
            0 => Ok(()),
            error => Err(Errno::from_i32(-error))
        }
    }
}

impl Drop for UInputDevice {
    fn drop(&mut self) {
        unsafe {
            raw::libevdev_uinput_destroy(self.raw);
        }
    }
}
