// ANT+ initialization and info SVCs.
// BASE=192: STACK_INIT=192, VERSION=235, CAPABILITIES=236, ENABLE=250

use super::to_asm;
use crate::ANT_ENABLE;

/// Reset ANT stack. Blocking, may timeout (~2s).
/// SVC 192
#[inline(always)]
pub unsafe fn sd_ant_stack_reset() -> u32 {
    let ret: u32;
    core::arch::asm!("svc 192",
        lateout("r0") ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Get ANT version string pointer.
/// SVC 235
#[inline(always)]
pub unsafe fn sd_ant_version(version: *mut u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 235",
        inout("r0") to_asm(version) => ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Get ANT capabilities.
/// SVC 236
#[inline(always)]
pub unsafe fn sd_ant_capabilities(capabilities: *mut u8) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 236",
        inout("r0") to_asm(capabilities) => ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}

/// Enable ANT with channel configuration.
/// SVC 250
#[inline(always)]
pub unsafe fn sd_ant_enable(config: *mut ANT_ENABLE) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 250",
        inout("r0") to_asm(config) => ret,
        lateout("r1") _,
        lateout("r2") _,
        lateout("r3") _,
        lateout("r12") _,
    );
    ret
}
