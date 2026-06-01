// ANT+ channel control SVCs.
// SVC numbers from ant_interface.h enum (BASE 0xC0 = 192):
//   194=ASSIGN, 195=UNASSIGN, 196=OPEN, 197=CLOSE, 198=RX_SCAN,
//   209=PERIOD_SET, 210=PERIOD_GET, 211=ID_SET, 212=ID_GET,
//   213=WAVEFORM_SET, 214=SEARCH_TIMEOUT, 215=PRIORITY, 218=LP_TIMEOUT

use super::to_asm;

/// Assign channel type and network.
/// SVC 194
#[inline(always)]
pub unsafe fn sd_ant_channel_assign(channel: u8, channel_type: u8, network: u8, ext_assign: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 194",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(channel_type) => _,
        inout("r2") to_asm(network) => _,
        inout("r3") to_asm(ext_assign) => _,
        lateout("r12") _,
    );
    ret
}

/// Unassign channel.
/// SVC 195
#[inline(always)]
pub unsafe fn sd_ant_channel_unassign(channel: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 195",
        inout("r0") to_asm(channel) => ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Open channel with start offset.
/// SVC 196
#[inline(always)]
pub unsafe fn sd_ant_channel_open_with_offset(channel: u8, offset: u16) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 196",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(offset) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Close channel.
/// SVC 197
#[inline(always)]
pub unsafe fn sd_ant_channel_close(channel: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 197",
        inout("r0") to_asm(channel) => ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Start RX scan mode. Channel 0 must be assigned, all others closed.
/// SVC 198
#[inline(always)]
pub unsafe fn sd_ant_rx_scan_mode_start(sync_channel_packets_only: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 198",
        inout("r0") to_asm(sync_channel_packets_only) => ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Set channel message period (32kHz counts, e.g. 8070 = ~4.06Hz).
/// SVC 209
#[inline(always)]
pub unsafe fn sd_ant_channel_period_set(channel: u8, period: u16) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 209",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(period) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Get channel message period.
/// SVC 210
#[inline(always)]
pub unsafe fn sd_ant_channel_period_get(channel: u8, period: *mut u16) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 210",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(period) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Set channel device ID.
/// SVC 211
#[inline(always)]
pub unsafe fn sd_ant_channel_id_set(channel: u8, device_number: u16, device_type: u8, transmit_type: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 211",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(device_number) => _,
        inout("r2") to_asm(device_type) => _,
        inout("r3") to_asm(transmit_type) => _,
        lateout("r12") _,
    );
    ret
}

/// Get channel device ID.
/// SVC 212
#[inline(always)]
pub unsafe fn sd_ant_channel_id_get(channel: u8, device_number: *mut u16, device_type: *mut u8, transmit_type: *mut u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 212",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(device_number) => _,
        inout("r2") to_asm(device_type) => _,
        inout("r3") to_asm(transmit_type) => _,
        lateout("r12") _,
    );
    ret
}

/// Set search waveform.
/// SVC 213
#[inline(always)]
pub unsafe fn sd_ant_search_waveform_set(channel: u8, waveform: u16) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 213",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(waveform) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Set channel search timeout.
/// SVC 214
#[inline(always)]
pub unsafe fn sd_ant_channel_search_timeout_set(channel: u8, timeout: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 214",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(timeout) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Set search channel priority (0-7).
/// SVC 215
#[inline(always)]
pub unsafe fn sd_ant_search_channel_priority_set(channel: u8, priority: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 215",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(priority) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Set low priority RX search timeout (2.5s units).
/// SVC 218
#[inline(always)]
pub unsafe fn sd_ant_channel_low_priority_rx_search_timeout_set(channel: u8, timeout: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 218",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(timeout) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Set active search sharing cycles (0 = disable, otherwise N×channel period).
/// SVC 216
#[inline(always)]
pub unsafe fn sd_ant_active_search_sharing_cycles_set(channel: u8, cycles: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 216",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(cycles) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}
