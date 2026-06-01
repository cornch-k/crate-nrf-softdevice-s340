// Boot-time hardware teardown that needs raw register access.
// Application code must stay safe Rust (memory policy 12) — keep unsafe here.

use core::ptr::{read_volatile, write_volatile};

// ─── GPIO raw helpers ────────────────────────────────────────────────
//
// Used at boot for light_kind detection on P1.10 (sense pin shared with
// UARTE1 TX) and for LIGHT_PWR (P1.01) outside of embassy_nrf's Peri
// ownership (so the same pin can be read raw, then handed back to a
// driver that writes its own PIN_CNF).

const P0_BASE: usize = 0x5000_0000;
const P1_BASE: usize = 0x5000_0300;

const OUT_OFFSET:    usize = 0x504;
const OUTSET_OFFSET: usize = 0x508;
const OUTCLR_OFFSET: usize = 0x50C;
const IN_OFFSET:     usize = 0x510;
const PIN_CNF_OFFSET: usize = 0x700;

// PIN_CNF bit layout (nRF52840 GPIO).
const CNF_DIR_OUTPUT: u32 = 1;        // bit 0
const CNF_INPUT_DISCONNECT: u32 = 1 << 1;
const CNF_PULL_DOWN: u32 = 1 << 2;    // bits 2..3 = 01
const CNF_DRIVE_H0H1: u32 = 3 << 8;   // bits 8..10 = 011

fn port_base(port: u8) -> usize {
    match port {
        0 => P0_BASE,
        1 => P1_BASE,
        _ => panic!("gpio: invalid port"),
    }
}

/// Configure pin as input + internal pulldown.
/// Used at boot for P1.10 light_kind sampling: with the external board
/// either pulling up (U1) or down/floating (Normal/none), the internal
/// pulldown ensures Normal/floating reads as LOW.
pub fn pin_cfg_input_pulldown(port: u8, pin: u8) {
    let base = port_base(port);
    unsafe {
        write_volatile(
            (base + PIN_CNF_OFFSET + (pin as usize) * 4) as *mut u32,
            CNF_PULL_DOWN, // DIR=Input, INPUT=Connect, PULL=Down, DRIVE=S0S1
        );
    }
}

/// Configure pin as disconnected (input buffer off, no drive, no pull).
/// Call before handing the pin off to a peripheral driver (e.g. UARTE)
/// so the driver's own PIN_CNF write is the only authority.
pub fn pin_cfg_disconnect(port: u8, pin: u8) {
    let base = port_base(port);
    unsafe {
        write_volatile(
            (base + PIN_CNF_OFFSET + (pin as usize) * 4) as *mut u32,
            CNF_INPUT_DISCONNECT,
        );
    }
}

/// Configure pin as output + high-drive (H0H1), no pull. Initial level
/// is taken from the pin's OUT register (set before calling if needed).
/// Used for LIGHT_PWR (P1.01) so it can sink/source the external light
/// board's current draw without sagging.
pub fn pin_cfg_output_high_drive(port: u8, pin: u8) {
    let base = port_base(port);
    unsafe {
        write_volatile(
            (base + PIN_CNF_OFFSET + (pin as usize) * 4) as *mut u32,
            CNF_DIR_OUTPUT | CNF_INPUT_DISCONNECT | CNF_DRIVE_H0H1,
        );
    }
}

/// Read pin input level (HIGH = true). Pin must be in input mode.
pub fn pin_read(port: u8, pin: u8) -> bool {
    let base = port_base(port);
    unsafe {
        let v = read_volatile((base + IN_OFFSET) as *const u32);
        (v & (1 << pin)) != 0
    }
}

/// Drive output high (atomic OUTSET write — single bit).
pub fn pin_set(port: u8, pin: u8) {
    let base = port_base(port);
    unsafe {
        write_volatile((base + OUTSET_OFFSET) as *mut u32, 1 << pin);
    }
}

/// Drive output low (atomic OUTCLR write — single bit).
pub fn pin_clear(port: u8, pin: u8) {
    let base = port_base(port);
    unsafe {
        write_volatile((base + OUTCLR_OFFSET) as *mut u32, 1 << pin);
    }
}

// ─── UARTE0 boot teardown (PR 0a) ────────────────────────────────────

const UARTE0_BASE: usize = 0x4000_2000;

const TASKS_STOPRX:     usize = 0x004;
const TASKS_STOPTX:     usize = 0x00C;
const EVENTS_RXTO:      usize = 0x114;
const EVENTS_TXSTOPPED: usize = 0x158;
const ENABLE:           usize = 0x500;
const PSEL_RTS:         usize = 0x508;
const PSEL_TXD:         usize = 0x50C;
const PSEL_CTS:         usize = 0x510;
const PSEL_RXD:         usize = 0x514;

const PSEL_DISCONNECTED: u32 = 0xFFFF_FFFF;
const ENABLE_UARTE:      u32 = 8;

#[inline(always)]
unsafe fn r32(off: usize) -> u32 {
    read_volatile((UARTE0_BASE + off) as *const u32)
}

#[inline(always)]
unsafe fn w32(off: usize, v: u32) {
    write_volatile((UARTE0_BASE + off) as *mut u32, v);
}

// ─── APPROTECT workaround for AAD0 build code ───────────────────────
//
// nRF52840 Engineering D (build code 'D', AAD0) is subject to Errata 248:
// APPROTECT is re-enabled by hardware on every reset, so the debug interface
// locks after each boot unless firmware explicitly writes APPROTECT.DISABLE.
//
// embassy-nrf 0.9 only runs this workaround for build_code >= 'F' (AAF0+),
// so AAD0 falls through and stays locked — flashing requires `nrfjprog
// --recover` every time. Pair this call with UICR.APPROTECT = 0x5A
// (eTrimm memory.x `.uicr_approtect` section) for a permanent fix.

const APPROTECT_DISABLE_REG: usize = 0x4000_0558;
const APPROTECT_DISABLE_SW_UNPROTECTED: u32 = 0x0000_005A;

/// Unlock the debug access port for chips where embassy-nrf's automatic
/// workaround does not apply (AAD0 build code on nRF52840). Call once at
/// the very start of `main`, before `embassy_nrf::init`.
pub fn approtect_disable_for_engd() {
    unsafe {
        write_volatile(
            APPROTECT_DISABLE_REG as *mut u32,
            APPROTECT_DISABLE_SW_UNPROTECTED,
        );
    }
}

// Bootloader (boot_ssd) jumps with UARTE0 enabled and PSEL.TXD = P1.10.
// PSEL silently overrides PIN_CNF, so light_kind PULLDOWN sampling on P1.10
// always reads HIGH unless we tear UARTE0 down first.
//
// Sequence: STOPTX → wait TXSTOPPED, STOPRX → wait RXTO, ENABLE = 0,
// then disconnect every PSEL.
pub fn uarte0_disable_for_boot() {
    unsafe {
        if r32(ENABLE) == ENABLE_UARTE {
            w32(EVENTS_TXSTOPPED, 0);
            w32(EVENTS_RXTO, 0);

            w32(TASKS_STOPTX, 1);
            let mut spins = 0u32;
            while r32(EVENTS_TXSTOPPED) == 0 && spins < 100_000 {
                spins += 1;
            }

            w32(TASKS_STOPRX, 1);
            spins = 0;
            while r32(EVENTS_RXTO) == 0 && spins < 100_000 {
                spins += 1;
            }

            w32(ENABLE, 0);
        }

        // Force every PSEL to disconnected even if UARTE0 was already off,
        // because the bootloader may have written PSEL without enabling.
        w32(PSEL_RTS, PSEL_DISCONNECTED);
        w32(PSEL_TXD, PSEL_DISCONNECTED);
        w32(PSEL_CTS, PSEL_DISCONNECTED);
        w32(PSEL_RXD, PSEL_DISCONNECTED);
    }
}
