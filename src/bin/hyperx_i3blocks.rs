use std::thread;
use std::{io, sync::mpsc::TryRecvError, time::Duration};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
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
                println!("");
                *device = pair_device();
            } else if message.contains("device disconnected") {
                eprintln!("Device disconnected.");
                println!("");
                *device = pair_device();
            } else {
                eprintln!("HID ERROR: {message}");
                std::process::exit(1);
            }
        }
        DeviceError::NoDeviceFound() => {
            eprintln!("{}", DeviceError::NoDeviceFound());
            device.clear_state();
            *device = pair_device();
        }
        error => {
            eprintln!("{error}");
            device.clear_state();
            *device = pair_device();
        }
    }
}

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });
    rx
}

fn main() {
    let mut device = pair_device();
    let stdin_channel = spawn_stdin_channel();

    // Run loop
    loop {
        match device.wait_for_updates(Duration::from_secs(1)) {
            Ok(_) => {}
            
            Err(DeviceError::UnknownResponse(_, _)) => (),
            Err(DeviceError::UnknownCommand(_)) => (),
            Err(DeviceError::NoResponse()) => (),

            Err(err) => handle_error(err, &mut device),

            
        }

        match device.headset_connected {
            Some(connected) => {
                if !connected {
                    println!("");
                    continue;
                }
            },
            None => {continue;}
        }

        let mut mute = false;
        match stdin_channel.try_recv() {
            Ok(key) => mute = key.contains("1"),
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
            Err(TryRecvError::Empty) => {},
        }
        
        match device.mic_connected {
            Some(connected) =>{
                if !connected {
                    println!(" - {}% ",device.battery_level);
                    continue;
                }
            },
            None => continue,
        }


        match device.muted {
            Some(muted) => {
                if muted {
                    if mute{
                        device.mute_mic(false);
                    }
                    println!(" - {}% ",device.battery_level);
                    
                } else {
                    if mute{
                        device.mute_mic(true);
                    }
                    println!(" - {}% ",device.battery_level);
                }
            },
            None => {println!(" - {}% ",device.battery_level);},
        }



    }
}
