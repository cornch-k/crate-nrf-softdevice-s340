// ANT+ data send/receive SVCs.
// ant_interface.h: SVC_ANT_EVENT_GET(193), SVC_ANT_TX_*(199-200), SVC_ANT_BURST(201-203)

use super::to_asm;

/// Get ANT event (channel number, event code, message buffer).
/// Buffer must be at least MESG_BUFFER_SIZE (41 bytes).
/// SVC 193
#[inline(always)]
pub unsafe fn sd_ant_event_get(channel: *mut u8, event: *mut u8, msg: *mut u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 193",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(event) => _,
        inout("r2") to_asm(msg) => _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Send broadcast message (8 bytes).
/// SVC 199
#[inline(always)]
pub unsafe fn sd_ant_broadcast_message_tx(channel: u8, size: u8, msg: *mut u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 199",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(size) => _,
        inout("r2") to_asm(msg) => _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Send acknowledged message (8 bytes).
/// SVC 200
#[inline(always)]
pub unsafe fn sd_ant_acknowledge_message_tx(channel: u8, size: u8, msg: *mut u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 200",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(size) => _,
        inout("r2") to_asm(msg) => _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Queue burst data. Size must be divisible by 8.
/// SVC 201
#[inline(always)]
pub unsafe fn sd_ant_burst_handler_request(channel: u8, size: u16, data: *mut u8, burst_segment: u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 201",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(size) => _,
        inout("r2") to_asm(data) => _,
        inout("r3") to_asm(burst_segment) => _,
        lateout("r12") _,
    );
    ret
}

/// Clear pending transmit on channel.
/// SVC 202
#[inline(always)]
pub unsafe fn sd_ant_pending_transmit_clear(channel: u8, success: *mut u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 202",
        inout("r0") to_asm(channel) => ret,
        inout("r1") to_asm(success) => _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Stop ongoing receive transfer.
/// SVC 203
#[inline(always)]
pub unsafe fn sd_ant_transfer_stop() -> u32 {
    let ret: u32;
    core::arch::asm!("svc 203",
        lateout("r0") ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}
