use hyper_x_cloud_ii_core_wireless::{Device, DeviceError};
use std::{thread, time::Duration};
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    mute_mic: Option<bool>,

    #[arg(short = 's', long)]
    monitor_mic: Option<bool>,

    #[arg(short = 't', long)]
    set_timeout: Option<u8>,

    #[arg(short = 'v' , long)]
    monitor_volume: Option<i8>,


    #[arg(short , long)]
    print_device: bool,



}
fn main() {
    let args: Args = Args::parse();
    let mut device = match Device::new() {
        Ok(device) => device,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };



    match args.monitor_volume {
        Some(volume) => {
            match device.set_monitor_volume(volume) {
                Ok(_) => {},
                Err(_) => {},
            }
        },
        None => {}
    }

    match args.set_timeout {
        Some(timeout) => {
            match device.set_timeout(timeout) {
                Ok(_) => {},
                Err(_) => {},
            }
        },
        None => {}
    }

    match args.mute_mic {
        Some(mute) => {
            match device.mute_mic(mute) {
                Ok(_) => {},
                Err(_) => {},
            }
        },
        None => {}
    }

    match args.monitor_mic {
        Some(monitor) => {
            match device.monitor_mic(monitor) {
                Ok(_) => {},
                Err(e) => {println!("{:?}", e)},
            }
        },
        None => {}
    }

    if args.print_device {
        for _ in 0..8 {
            match device.wait_for_updates(Duration::from_secs(1)) {
                Ok(_) => {
    
                }
                Err(error) => {
                    println!("{:?}", error);
                    
                }
            }
        }
        println!("{}", device);


    }
    
   

}

#[test]
fn test_basic_device_access() {
    let _ = match Device::new() {
        Ok(device) => device,
        Err(_) => return
    };
}