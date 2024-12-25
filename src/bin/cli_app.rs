use clap::{Parser, Subcommand};
use hyper_x_cloud_ii_core_wireless::{Device};
use std::time::Duration;
#[derive(Parser, Debug)]
#[clap(
    about = " A CLI tool to control HyperX Cloud II Core Wireless device."
)]
struct Args {
    #[command(
        subcommand,
    )]
    op: Operation,
}

#[derive(Subcommand, Debug)]
enum Operation {
    #[command(
        arg_required_else_help = true,
        about = "Set device settings."
    )]
    Set {
        #[arg(
            short,
            long,
            help="Mute the microphone. True to mute, false to unmute."
        )]
        mute_mic: Option<bool>,

        #[arg(
            short = 's',
            long,
            help="Monitor the microphone. True to monitor, false to stop monitoring."
        )]
        monitor_mic: Option<bool>,

        #[arg(
            short = 't',
            long,
            help="The device will automatically turn off after this many minutes of inactivity. 0 to disable."
        )]
        timeout: Option<u8>,

        #[arg(
            short = 'v',
            long,
            help="Monitor volume, -5 to 5."
        )]
        monitor_volume: Option<i8>,
    },
    #[command(
        arg_required_else_help = true,
        about="Get device settings."
    )]
    Get {
        #[arg(
            short,
            long,
            help="Print human readable report "
        )]
        print_device: bool,

        #[arg(
            short,
            long,
            value_name="PATTERN",
            help="Generate a report based on the pattern. Possible values \"mstvCcbM\"
    m for mute
    s for monitor
    t for timeout
    v for volume
    C for headset connected
    c for charging
    b for battery level
    M for mic connected"
        )]
        generate_report: Option<String>,
    },
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

    match args.op {
        Operation::Set {
            mute_mic,
            monitor_mic,
            timeout,
            monitor_volume,
        } => {
            match timeout {
                Some(timeout) => match device.set_timeout(timeout) {
                    Ok(_) => {}
                    Err(_) => {}
                },
                None => {}
            }

            match mute_mic {
                Some(mute) => match device.mute_mic(mute) {
                    Ok(_) => {}
                    Err(_) => {}
                },
                None => {}
            }

            match monitor_mic {
                Some(monitor) => match device.monitor_mic(monitor) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("{:?}", e)
                    }
                },
                None => {}
            }

            match monitor_volume {
                Some(volume) => match device.set_monitor_volume(volume) {
                    Ok(_) => {}
                    Err(_) => {}
                },
                None => {}
            }
        }
        Operation::Get {
            print_device,
            generate_report,
        } => {
            for _ in 0..8 {
                match device.wait_for_updates(Duration::from_secs(1)) {
                    Ok(_) => {}
                    Err(error) => {
                        println!("{:?}", error);
                    }
                }
            }

            if print_device {
                println!("{}", device);
            }

            match generate_report {
                Some(pattern) => {
                    for get in pattern.chars() {
                        if get == 'm' {
                            match device.muted {
                                Some(status) => {
                                    print!("{} ", status);
                                }

                                None => {
                                    print!("N/A");
                                }
                            }
                        } else if get == 's' {
                            match device.mic_monitored {
                                Some(status) => {
                                    print!("{} ", status);
                                }

                                None => {
                                    print!("N/A");
                                }
                            }
                        } else if get == 't' {
                            print!("{} ", device.timeout);

                        } else if get == 'v' {
                            print!("{} ", device.monitor_volume);

                        } else if get == 'C' {
                            match device.headset_connected {
                                Some(status) => {
                                    print!("{} ", status);
                                }

                                None => {
                                    print!("N/A");
                                }
                            }
                        } else if get == 'c' {
                            match device.charging {
                                Some(status) => {
                                    print!("{} ", status);
                                }

                                None => {
                                    print!("N/A");
                                }
                            }
                        } else if get == 'b' {
                            print!("{} ", device.battery_level);

                        } else if get == 'M' {
                            match device.mic_connected {
                                Some(status) => {
                                    print!("{} ", status);
                                }

                                None => {
                                    print!("N/A");
                                }
                            }
                        }
                    }
                }
                None => {}
            }
        }
    }
}

#[test]
fn test_basic_device_access() {
    let _ = match Device::new() {
        Ok(device) => device,
        Err(_) => return,
    };
}
