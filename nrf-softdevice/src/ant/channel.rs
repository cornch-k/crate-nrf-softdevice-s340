// ANT+ Channel — typed wrapper for channel operations.
// C original: ant_channel_open/close/register in ratio_ant_scanning.c

use crate::raw;
use crate::RawError;

/// ANT channel configuration.
pub struct AntChannelConfig {
    pub channel_type: u8,
    pub network: u8,
    pub ext_assign: u8,
    pub rf_freq: u8,
    pub device_number: u16,
    pub device_type: u8,
    pub transmit_type: u8,
    pub period: u16,
    pub search_timeout: u8,
    pub low_priority_timeout: u8,
}

impl Default for AntChannelConfig {
    fn default() -> Self {
        Self {
            channel_type: 0x00, // Slave (receive)
            network: 0,
            ext_assign: 0,
            rf_freq: 57, // ANT+ standard: 2457MHz
            device_number: 0, // Wildcard
            device_type: 0,   // Wildcard
            transmit_type: 0, // Wildcard
            period: 8070,     // Default (~4Hz)
            search_timeout: 255, // Infinite
            low_priority_timeout: 255, // Infinite
        }
    }
}

/// A configured ANT channel.
pub struct AntChannel {
    pub num: u8,
}

impl AntChannel {
    /// Assign and configure a channel. Does not open it yet.
    pub fn configure(channel_num: u8, config: &AntChannelConfig) -> Result<Self, RawError> {
        unsafe {
            // Assign channel type + network.
            let ret = raw::ant::channel::sd_ant_channel_assign(
                channel_num, config.channel_type, config.network, config.ext_assign,
            );
            RawError::convert(ret)?;

            // Set RF frequency.
            let ret = raw::ant::config::sd_ant_channel_radio_freq_set(channel_num, config.rf_freq);
            RawError::convert(ret)?;

            // Set device ID (number, type, transmit type).
            let ret = raw::ant::channel::sd_ant_channel_id_set(
                channel_num, config.device_number, config.device_type, config.transmit_type,
            );
            RawError::convert(ret)?;

            // Set message period.
            let ret = raw::ant::channel::sd_ant_channel_period_set(channel_num, config.period);
            RawError::convert(ret)?;

            // Set search timeouts.
            let ret = raw::ant::channel::sd_ant_channel_search_timeout_set(channel_num, config.search_timeout);
            RawError::convert(ret)?;

            let ret = raw::ant::channel::sd_ant_channel_low_priority_rx_search_timeout_set(
                channel_num, config.low_priority_timeout,
            );
            RawError::convert(ret)?;
        }

        Ok(Self { num: channel_num })
    }

    /// Open the channel and start searching/transmitting.
    pub fn open(&self) -> Result<(), RawError> {
        let ret = unsafe {
            raw::ant::channel::sd_ant_channel_open_with_offset(self.num, 0)
        };
        RawError::convert(ret)
    }

    /// Close the channel.
    pub fn close(&self) -> Result<(), RawError> {
        let ret = unsafe { raw::ant::channel::sd_ant_channel_close(self.num) };
        RawError::convert(ret)
    }

    /// Unassign the channel.
    pub fn unassign(&self) -> Result<(), RawError> {
        let ret = unsafe { raw::ant::channel::sd_ant_channel_unassign(self.num) };
        RawError::convert(ret)
    }

    /// Get channel number.
    pub fn number(&self) -> u8 {
        self.num
    }

    /// Get channel status.
    pub fn status(&self) -> Result<u8, RawError> {
        let mut status: u8 = 0;
        let ret = unsafe { raw::ant::status::sd_ant_channel_status_get(self.num, &mut status) };
        RawError::convert(ret)?;
        Ok(status)
    }

    /// Send broadcast data (8 bytes).
    pub fn broadcast(&self, data: &mut [u8; 8]) -> Result<(), RawError> {
        let ret = unsafe {
            raw::ant::data::sd_ant_broadcast_message_tx(self.num, 8, data.as_mut_ptr())
        };
        RawError::convert(ret)
    }

    /// Set search waveform (수신 윈도우 주기). 97=Fast, 316=Default.
    pub fn set_search_waveform(&self, waveform: u16) -> Result<(), RawError> {
        let ret = unsafe {
            raw::ant::channel::sd_ant_search_waveform_set(self.num, waveform)
        };
        RawError::convert(ret)
    }

    /// Send acknowledged data (8 bytes). Slave → Master 전송 시 사용.
    pub fn acknowledge(&self, data: &mut [u8; 8]) -> Result<(), RawError> {
        let ret = unsafe {
            raw::ant::data::sd_ant_acknowledge_message_tx(self.num, 8, data.as_mut_ptr())
        };
        RawError::convert(ret)
    }
}
