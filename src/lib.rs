use std::{time::Duration};

use hidapi::{HidApi, HidDevice, HidError};
use thistermination::{TerminationFull};

use num_enum::{TryFromPrimitive, TryFromPrimitiveError};

// Possible vendor IDs [hyperx , HP]
const VENDOR_IDS: [u16; 2] = [0x0951, 0x03F0];
// Possible Cloud II Core Wireless product IDs
const PRODUCT_IDS: [u16; 1] = [0x0995];


const MAGIC_BYTE: u8 = 102;

#[derive(Debug, Eq, Copy, Clone, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum ReportByte {
    SetMonitorState = 1,
    SetPowerAutoOffTiming = 2,
    SetMicMuteState = 3,
    //SetPlaybackMuteState = 4,
    SetMonitorVolume = 5,
    UpdateMicConnectionStatus = 7,
    UpdateMicMonitorStatus = 9,
    UpdateMicMuteStatus = 10,
    UpdateConnectedStatus = 11,
    UpdateChargingStatus = 12,
    UpdateBatteryStatus = 13,
    //SetNoiseGateState = 15,
    //GetDeviceInformation = 129,
    GetConnectedStatus = 130,
    GetMonitorState = 132,
    GetPowerAutoOffTiming = 133,
    GetMicMuteState = 134,
    //GetPlaybackMuteState = 135,
    GetMonitorVolume = 136,
    GetBatteryLevel = 137,
    GetChargerState = 138,
    GetMicPlugState = 140,
    //GetNoiseGateState = 141,
}

impl From<TryFromPrimitiveError<ReportByte>> for DeviceError {
    fn from(err: TryFromPrimitiveError<ReportByte>) -> DeviceError {
        DeviceError::UnknownCommand(err.number)
    }
}


#[derive(Debug)]
pub enum DeviceEvent {
    MicConnected(bool),
    MonitoringMic(bool),
    MicMuted(bool),
    HeadsetConnected(bool),
    Charging(bool),
    ChargeLevel(u8),
    SetTimeout(u8),
    GetTimeout(u8),
    SetBatteryLevel(u8),
    MonitorVolume(u8),
    SetMonitorVolume(u8),
}

impl DeviceEvent {
    pub fn get_event_from_buf(buf: &[u8; 8], len: usize) -> Result<Self, DeviceError> {
        if len == 0 {
            return Err(DeviceError::NoResponse());
        }
        if len != 8 {
            return Err(DeviceError::UnknownResponse(buf.clone(), len));
        }

        let original_buf = buf;

        let (&byte, buf) = buf.split_first().unwrap();
        if byte != MAGIC_BYTE {
            return Err(DeviceError::UnknownResponse(original_buf.clone(), len));
        }

        let (&byte, buf) = buf.split_first().unwrap();
        let command = ReportByte::try_from(byte).map_err(|err| {
            dbg!(&original_buf);
            err
        })?;

        match command {
            ReportByte::SetMonitorState => {Ok(Self::MonitoringMic(buf[0] == 1))},
            ReportByte::SetPowerAutoOffTiming => Ok(Self::SetTimeout(buf[0])),
            ReportByte::SetMicMuteState => Ok(Self::MicMuted(buf[0] == 1)),
            ReportByte::SetMonitorVolume => {Ok(Self::SetMonitorVolume(buf[0]))},
            ReportByte::UpdateMicConnectionStatus => { Ok(Self::MicConnected(buf[0] == 1)) },
            ReportByte::UpdateMicMonitorStatus => { Ok(Self::MonitoringMic(buf[0] == 1)) },
            ReportByte::UpdateMicMuteStatus => { Ok(Self::MicMuted(buf[0] == 1)) },
            ReportByte::UpdateConnectedStatus => { Ok(Self::HeadsetConnected(buf[0] == 1))},
            ReportByte::UpdateChargingStatus => { Ok(Self::Charging(buf[0] == 1))},
            ReportByte::UpdateBatteryStatus => { Ok(Self::ChargeLevel(buf[2]))},
            ReportByte::GetMonitorState => {Ok(Self::MonitoringMic(buf[0] == 1))},
            ReportByte::GetPowerAutoOffTiming => {Ok(Self::GetTimeout(buf[0]))},
            ReportByte::GetMicMuteState => {Ok(Self::MicMuted(buf[0] == 1))},
            ReportByte::GetMonitorVolume => {Ok(Self::MonitorVolume(buf[0]))},
            ReportByte::GetBatteryLevel =>{Ok(Self::SetBatteryLevel(buf[2]))},
            ReportByte::GetChargerState => {Ok(Self::Charging(buf[0] == 1))},
            ReportByte::GetMicPlugState => {Ok(Self::MicConnected(buf[0] == 1))},
            ReportByte::GetConnectedStatus => {Ok(Self::HeadsetConnected(buf[0] == 1))},
        }

    }
}

#[derive(TerminationFull)]
pub enum DeviceError {
    #[termination(msg("{0:?}"))]
    HidError(#[from] HidError),
    #[termination(msg("No device found."))]
    NoDeviceFound(),
    #[termination(msg("No response."))]
    NoResponse(),
    #[termination(msg("Unknown response: {0:?} with length: {1:?}"))]
    UnknownResponse([u8; 8], usize),
    #[termination(msg("Unknown command: {0}"))]
    UnknownCommand(u8),
}

#[derive(Debug)]
pub struct Device {
    hid_device: HidDevice,
    pub headset_connected: Option<bool>,
    pub battery_level: u8,
    pub charging: Option<bool>,
    pub mic_connected: Option<bool>,
    pub muted: Option<bool>,
    pub mic_monitored: Option<bool>,
    pub timeout: u8,
    pub monitor_volume: u8,

}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Headset Connected {:?}\nBattery: {}\nCharging: {:?}\nMic Connected: {:?}\nMic Muted: {:?}\nMonitor On: {:?}\nMonitor Volume:{}\nIdle Timeout: {}\nMonitor Volume: {}",
            self.headset_connected, self.battery_level, self.charging, self.mic_connected, self.muted, self.mic_monitored, self.monitor_volume, self.timeout, self.monitor_volume
        )
    }
}
impl Device {
    pub fn new() -> Result<Self, DeviceError> {
        let hid_api = HidApi::new()?;
        let hid_device = hid_api
            .device_list()
            .find_map(|info| {
                if PRODUCT_IDS.contains(&info.product_id())
                    && VENDOR_IDS.contains(&info.vendor_id())
                {
                    Some(hid_api.open(info.vendor_id(), info.product_id()))
                } else {
                    None
                }
            })
            .ok_or(DeviceError::NoDeviceFound())??;
        let device = Device {
            hid_device,
            headset_connected: None,
            battery_level: 0,
            charging: None,
            mic_connected: None,
            muted: None,
            mic_monitored: None,
            timeout: 0,
            monitor_volume: 0,
        };

        device.sync_state();
        Ok(device)
    }

    fn update_self_with_event(&mut self, event: &DeviceEvent) {
        
        match event {
            DeviceEvent::MicConnected(connected) => self.mic_connected = Some(*connected),
            DeviceEvent::MonitoringMic(monitoring) => self.mic_monitored = Some(*monitoring),
            DeviceEvent::MicMuted(muted) => self.muted = Some(*muted),
            DeviceEvent::HeadsetConnected(connected) => self.headset_connected = Some(*connected),
            DeviceEvent::Charging(charging) => self.charging = Some(*charging),
            DeviceEvent::ChargeLevel(level) => self.battery_level = level.clone(),
            DeviceEvent::GetTimeout(timeout) => self.timeout = timeout.clone(),
            DeviceEvent::SetTimeout(timeout) => self.timeout = timeout.clone(),
            DeviceEvent::SetBatteryLevel(level) => self.battery_level = *level,
            DeviceEvent::MonitorVolume(volume) => self.monitor_volume = *volume,
            DeviceEvent::SetMonitorVolume(volume) => self.monitor_volume = *volume,
        };

    }

    pub fn wait_for_updates(&mut self, duration: Duration) -> Result<DeviceEvent, DeviceError> {
        let mut buf: [u8; 8] = [0u8; 8];
        let res = self
            .hid_device
            .read_timeout(&mut buf[..], duration.as_millis() as i32)?;
        match DeviceEvent::get_event_from_buf(&buf, res) {
            Ok(event) => {
                self.update_self_with_event(&event);
                Ok(event)
            }
            Err(error) => Err(error),
        }
    }

    pub fn mute_mic(&self, mute: bool) -> Result<usize, HidError> {
        return self.hid_device.write(&[MAGIC_BYTE, ReportByte::SetMicMuteState as u8,mute as u8]);
    }

    pub fn monitor_mic(&self, mute: bool)-> Result<usize, HidError>{
        return self.hid_device.write(&[MAGIC_BYTE, ReportByte::SetMonitorState as u8, mute as u8]);
    }
    
    pub fn set_timeout(&self, timeout: u8) -> Result<usize, HidError> {
        return self.hid_device.write(&[MAGIC_BYTE, ReportByte::SetPowerAutoOffTiming as u8, timeout]);
    }
    
    pub fn set_monitor_volume(&self, volume: i8) -> Result<usize, HidError> {
        //TODO must be -5 <= volume <= 5
        return self.hid_device.write(&[MAGIC_BYTE, ReportByte::SetMonitorVolume as u8, volume.to_ne_bytes()[0]]);
    }
    
    pub fn update_battery_level(&self) -> Result<usize, HidError> {
        
        self.hid_device.write(&[MAGIC_BYTE, ReportByte::GetBatteryLevel as u8])
    }
    
    pub fn get_monitor_volume(&self) -> Result<usize, HidError> {
        self.hid_device.write(&[MAGIC_BYTE, ReportByte::GetMonitorVolume as u8])
    }

    pub fn get_timeout(&self) -> Result<usize, HidError> {
        self.hid_device.write(&[MAGIC_BYTE, ReportByte::GetPowerAutoOffTiming as u8])
    }
    
    pub fn get_monitor_state(&self) -> Result<usize, HidError> {
        self.hid_device.write(&[MAGIC_BYTE, ReportByte::GetMonitorState as u8])
    }

    pub fn get_mic_mute_state(&self) -> Result<usize, HidError> {
        self.hid_device.write(&[MAGIC_BYTE, ReportByte::GetMicMuteState as u8])
    }

    pub fn get_charger_state(&self) -> Result<usize, HidError> {
        self.hid_device.write(&[MAGIC_BYTE, ReportByte::GetChargerState as u8])
    }

    pub fn get_mic_connected(&self) -> Result<usize, HidError> {
        self.hid_device.write(&[MAGIC_BYTE, ReportByte::GetMicPlugState as u8])
    }

    pub fn get_headset_connected(&self) -> Result<usize, HidError> {
        self.hid_device.write(&[MAGIC_BYTE, ReportByte::GetConnectedStatus as u8])
    }

    pub fn sync_state(&self) {
        self.get_headset_connected();
        self.update_battery_level();
        self.get_timeout();
        self.get_mic_mute_state();
        self.get_monitor_state();
        self.get_monitor_volume();
        self.get_charger_state();
        self.get_mic_connected();
        

    }
    pub fn clear_state(&mut self) {
        self.headset_connected = None;
        self.battery_level = 0;
        self.charging = None;
        self.mic_connected = None;
        self.muted = None;
        self.mic_monitored = None;
        self.timeout = 0;
        self.monitor_volume = 0;
    }
}
