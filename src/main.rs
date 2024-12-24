use std::time::Duration;

use hyper_x_cloud_ii_core_wireless::{Device, DeviceError};


fn pair_device() -> Device {
    loop {
        match Device::new() {
            Ok(device) => break device,
            Err(error) => {
                eprintln!("{error}");
            }
        };
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn handle_error(error: DeviceError, device: &mut Device) {

    match error {
        DeviceError::HidError(hidapi::HidError::HidApiError { message }) => {
            if message == "No such device" {
                eprintln!("No device found.");
                *device = pair_device();
            } else if message.contains("device disconnected") {
                eprintln!("Device disconnected.");
                *device = pair_device();
            } 
            else {
                eprintln!("HID ERROR: {message}");
                std::process::exit(1);            }
        }
        DeviceError::NoDeviceFound() => {
            eprintln!("{}", DeviceError::NoDeviceFound());
            device.clear_state();
                   }
        error => {
            eprintln!("{error}");
        }
    }
}

fn main() {
    let mut device = pair_device();

    // Run loop
    loop {
        match device.wait_for_updates(Duration::from_secs(10)) {
            Ok(_) => {
                
            }
            Err(DeviceError::NoResponse()) => (),
            Err(DeviceError::UnknownResponse(_, _)) => (),
            Err(error) => {
                continue;
            }
        }

        print!("{esc}c", esc = 27 as char);
        print!("{}", device);
    }
}
