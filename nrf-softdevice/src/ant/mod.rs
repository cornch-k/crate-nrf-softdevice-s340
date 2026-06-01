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

use core::sync::atomic::{AtomicUsize, Ordering};

use static_cell::StaticCell;

use crate::raw;
use crate::RawError;

/// ANT+ network number for standard ANT+ devices.
pub const ANTPLUS_NETWORK_NUM: u8 = 0;

/// Memory required per ANT channel (from ant_parameters.h).
/// ANT_ENABLE_GET_REQUIRED_SPACE(channels, encrypted, num_events, burst_size)
/// = channels * 616 + encrypted * 64 + num_events * EVENT_SIZE + burst_size + 488
/// 2026-05-20: 자전거 페어링 시 APP_MEMACC fault 빈발. NUM_EVENTS 분 (default ~6 * 24 = 144)
/// 미반영 + burst queue 너무 작음. 보수적으로 base 1500, burst 256 으로 늘림.
const ANT_CHANNEL_SIZE: usize = 616;
const ANT_BASE_SIZE: usize = 1500;
const ANT_BURST_QUEUE_SIZE: usize = 256;

// 4-byte aligned wrapper — SD ANT 가 word-aligned buffer 요구할 수 있음.
// 2026-05-20: 16KB 는 .bss overflow / stack 부족 → 부팅 실패. 12KB 로 축소.
#[repr(align(4))]
struct AntMemBuf([u8; 12288]);

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
    // 15 channels * 616 + 1500 + 256 = 10996 bytes. Buffer 12KB.
    // 2026-05-20: [0u8; N] literal 은 stack 에 12KB 임시 객체 만들어서 stack
    // overflow (SD reserved RAM 침범 → APP_MEMACC). MaybeUninit 으로 static
    // 에서 직접 init — zero-fill 불필요 (SD ANT 가 자기 데이터로 덮음).
    use core::mem::MaybeUninit;
    static ANT_MEM: StaticCell<MaybeUninit<AntMemBuf>> = StaticCell::new();
    let buf_uninit = ANT_MEM.init(MaybeUninit::uninit());
    // SAFETY: SD ANT 가 sd_ant_enable 후 buffer 안만 access. 우리는 ptr/size 만 전달.
    let buf_ref = unsafe { buf_uninit.assume_init_mut() };
    let buf = &mut buf_ref.0;

    let mut config = raw::ANT_ENABLE {
        ucTotalNumberOfChannels: total_channels,
        ucNumberOfEncryptedChannels: encrypted_channels,
        usNumberOfEvents: 0, // Use default
        pucMemoryBlockStartLocation: buf.as_mut_ptr(),
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
// fn pointer를 usize로 보관 (0 = 미등록). `static mut` 회피.
static ANT_EVENT_HANDLER: AtomicUsize = AtomicUsize::new(0);

/// Register a global ANT event handler.
/// Must be called before Softdevice::run().
pub fn set_event_handler(handler: fn(&AntEvent)) {
    ANT_EVENT_HANDLER.store(handler as usize, Ordering::Release);
}

/// Dispatch ANT event to registered handler. Called from softdevice run loop.
pub(crate) fn dispatch_event(evt: &AntEvent) {
    let raw = ANT_EVENT_HANDLER.load(Ordering::Acquire);
    if raw != 0 {
        // SAFETY: set_event_handler만이 이 값을 저장하며, 항상 유효한 `fn(&AntEvent)` ptr만 저장함.
        let handler: fn(&AntEvent) = unsafe { core::mem::transmute(raw) };
        handler(evt);
    }
}
