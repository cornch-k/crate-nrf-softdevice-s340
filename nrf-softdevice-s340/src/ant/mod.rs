// ANT+ SoftDevice SVC bindings.
// Ported from softdevice/s340/headers/ant_interface.h
//
// SVC base: STK_SVC_BASE_2 = 0xC0 (192)
// SVC numbers follow the enum order in ant_interface.h.

pub mod channel;
pub mod config;
pub mod data;
pub mod init;
pub mod status;

// to_asm helper — same as bindings.rs but local to this module.
trait ToAsm {
    fn to_asm(self) -> u32;
}

fn to_asm<T: ToAsm>(t: T) -> u32 {
    t.to_asm()
}

impl ToAsm for u32 {
    fn to_asm(self) -> u32 { self }
}
impl ToAsm for u16 {
    fn to_asm(self) -> u32 { self as u32 }
}
impl ToAsm for u8 {
    fn to_asm(self) -> u32 { self as u32 }
}
impl<T> ToAsm for *const T {
    fn to_asm(self) -> u32 { self as u32 }
}
impl<T> ToAsm for *mut T {
    fn to_asm(self) -> u32 { self as u32 }
}
