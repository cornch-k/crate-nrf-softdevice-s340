// ANT+ radio and library configuration SVCs.
// BASE=192. Exact SVC numbers from ant_interface.h enum order:
//   204=NETWORK_KEY, 205=FREQ_SET, 206=FREQ_GET, 207=TX_POWER, 208=PROX_SEARCH
//   219=ADV_BURST_SET, 220=ADV_BURST_GET
//   221=LIB_CONFIG_SET, 222=LIB_CONFIG_CLEAR, 223=LIB_CONFIG_GET
//   228=EVENT_FILTERING_SET, 229=EVENT_FILTERING_GET (wait — ACTIVE=229...)
// Recount: FILTERING_SET=228, FILTERING_GET=229? No, ACTIVE=229.
// Let me recount from 192:
//   26=LP_TIMEOUT(218), 27=ADV_BURST_SET(219), 28=ADV_BURST_GET(220),
//   29=LIB_CONFIG_SET(221), 30=LIB_CONFIG_CLEAR(222), 31=LIB_CONFIG_GET(223),
//   32=ID_LIST_ADD(224), 33=ID_LIST_CONFIG(225), 34=AUTO_FREQ_HOP(226),
//   35=EVENT_FILTERING_SET(227), 36=EVENT_FILTERING_GET(228),
//   37=ACTIVE(229)
// So FILTERING_SET=227, FILTERING_GET=228.

use super::to_asm;

/// Set 64-bit network key.
/// SVC 204
#[inline(always)]
pub unsafe fn sd_ant_network_address_set(network: u8, key: *const u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 204",
        inout("r0") to_asm(network) => ret,
        inout("r1") to_asm(key) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Set channel RF frequency (offset from 2400MHz).
/// SVC 205
#[inline(always)]
pub unsafe fn sd_ant_channel_radio_freq_set(channel: u8, freq: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 205",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(freq) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Get channel RF frequency.
/// SVC 206
#[inline(always)]
pub unsafe fn sd_ant_channel_radio_freq_get(channel: u8, freq: *mut u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 206",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(freq) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Set channel TX power.
/// SVC 207
#[inline(always)]
pub unsafe fn sd_ant_channel_radio_tx_power_set(channel: u8, tx_power: u8, custom_tx_power: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 207",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(tx_power) => _,
        inout("r2") to_asm(custom_tx_power) => _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Set proximity search threshold.
/// SVC 208
#[inline(always)]
pub unsafe fn sd_ant_prox_search_set(channel: u8, prox_threshold: u8, custom_prox_threshold: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 208",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(prox_threshold) => _,
        inout("r2") to_asm(custom_prox_threshold) => _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Set ANT library config (RSSI, device ID in messages).
/// SVC 221
#[inline(always)]
pub unsafe fn sd_ant_lib_config_set(config: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 221",
        inout("r0") to_asm(config) => ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Clear ANT library config bits.
/// SVC 222
#[inline(always)]
pub unsafe fn sd_ant_lib_config_clear(config: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 222",
        inout("r0") to_asm(config) => ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Get ANT library config.
/// SVC 223
#[inline(always)]
pub unsafe fn sd_ant_lib_config_get(config: *mut u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 223",
        inout("r0") to_asm(config) => ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Set event filtering.
/// SVC 227
#[inline(always)]
pub unsafe fn sd_ant_event_filtering_set(filter: u16) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 227",
        inout("r0") to_asm(filter) => ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Get event filtering.
/// SVC 228
#[inline(always)]
pub unsafe fn sd_ant_event_filtering_get(filter: *mut u16) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 228",
        inout("r0") to_asm(filter) => ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Set coexistence config.
/// SVC 248
#[inline(always)]
pub unsafe fn sd_ant_coex_config_set(channel: u8, config: *mut crate::ANT_BUFFER_PTR) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 248",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(config) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Get coexistence config.
/// SVC 249
#[inline(always)]
pub unsafe fn sd_ant_coex_config_get(channel: u8, config: *mut crate::ANT_BUFFER_PTR) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 249",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(config) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}
