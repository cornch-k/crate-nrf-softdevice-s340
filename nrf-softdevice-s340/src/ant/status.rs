// ANT+ status query SVCs.
// BASE=192: ACTIVE=229, CHANNEL_IN_PROGRESS=230, CHANNEL_STATUS_GET=231, PENDING_TRANSMIT=232

use super::to_asm;

/// Check if ANT is active.
/// SVC 229
#[inline(always)]
pub unsafe fn sd_ant_active(ant_active: *mut u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 229",
        inout("r0") to_asm(ant_active) => ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Check if a channel is in progress.
/// SVC 230
#[inline(always)]
pub unsafe fn sd_ant_channel_in_progress(in_progress: *mut u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 230",
        inout("r0") to_asm(in_progress) => ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Get channel status.
/// SVC 231
#[inline(always)]
pub unsafe fn sd_ant_channel_status_get(channel: u8, status: *mut u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 231",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(status) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Check if there is a pending transmit on channel.
/// SVC 232
#[inline(always)]
pub unsafe fn sd_ant_pending_transmit(channel: u8, pending: *mut u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 232",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(pending) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}
