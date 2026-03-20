// ANT+ high-level API for nrf-softdevice.
// Provides typed wrappers around raw sd_ant_* SVC calls.
//
// Usage:
//   1. Call ant_enable() after SoftDevice is enabled
//   2. Softdevice::run() automatically polls ANT events
//   3. Use AntChannel to configure and open channels
//   4. Receive data via ant event callbacks

mod channel;
pub(crate) mod event;

pub use channel::*;
pub use event::*;

use crate::raw;
use crate::RawError;

/// ANT+ network number for standard ANT+ devices.
pub const ANTPLUS_NETWORK_NUM: u8 = 0;

/// Memory required per ANT channel (from ant_parameters.h).
/// ANT_ENABLE_GET_REQUIRED_SPACE(channels, encrypted, burst_size)
/// = channels * 616 + encrypted * 64 + 588 + burst_size
const ANT_CHANNEL_SIZE: usize = 616;
const ANT_BASE_SIZE: usize = 588;
const ANT_BURST_QUEUE_SIZE: usize = 128;

/// Enable ANT stack with the given number of channels.
/// Must be called after sd_softdevice_enable() and before any ANT operations.
///
/// C original: nrf_sdh_ant_enable() → sd_ant_enable()
pub fn ant_enable(total_channels: u8, encrypted_channels: u8) -> Result<(), RawError> {
    // Calculate required memory.
    let required = total_channels as usize * ANT_CHANNEL_SIZE
        + encrypted_channels as usize * 64
        + ANT_BASE_SIZE
        + ANT_BURST_QUEUE_SIZE;

    // Static buffer for ANT stack memory.
    // Max 15 channels * 616 + 588 + 128 = 9956 bytes.
    static mut ANT_MEM: [u8; 10240] = [0; 10240];

    let mut config = raw::ANT_ENABLE {
        ucTotalNumberOfChannels: total_channels,
        ucNumberOfEncryptedChannels: encrypted_channels,
        usNumberOfEvents: 0, // Use default
        pucMemoryBlockStartLocation: unsafe { ANT_MEM.as_mut_ptr() },
        usMemoryBlockByteSize: required as u16,
    };

    let ret = unsafe { raw::ant::init::sd_ant_enable(&mut config) };
    RawError::convert(ret)
}

/// Set ANT+ network key on the given network number.
/// Standard ANT+ key must be obtained from Garmin/ANT+ Alliance.
pub fn set_network_key(network: u8, key: &[u8; 8]) -> Result<(), RawError> {
    let ret = unsafe { raw::ant::config::sd_ant_network_address_set(network, key.as_ptr()) };
    RawError::convert(ret)
}

/// Configure ANT library to include RSSI and device ID in messages.
/// C original: sd_ant_lib_config_set(ANT_LIB_CONFIG_MESG_OUT_INC_RSSI | ANT_LIB_CONFIG_MESG_OUT_INC_DEVICE_ID)
pub fn set_lib_config(config: u8) -> Result<(), RawError> {
    let ret = unsafe { raw::ant::config::sd_ant_lib_config_set(config) };
    RawError::convert(ret)
}

// Global ANT event handler — set by application, called from Softdevice::run().
static mut ANT_EVENT_HANDLER: Option<fn(&AntEvent)> = None;

/// Register a global ANT event handler.
/// Must be called before Softdevice::run().
pub fn set_event_handler(handler: fn(&AntEvent)) {
    unsafe { ANT_EVENT_HANDLER = Some(handler); }
}

/// Dispatch ANT event to registered handler. Called from softdevice run loop.
pub(crate) fn dispatch_event(evt: &AntEvent) {
    unsafe {
        if let Some(handler) = ANT_EVENT_HANDLER {
            handler(evt);
        }
    }
}
