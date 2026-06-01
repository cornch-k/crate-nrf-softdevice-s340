//! `static mut` 회피용 safe 래퍼.
//! `UnsafeCell`에 `unsafe impl Sync`만 달아 static으로 선언 가능하게 함.
//! 내부 접근은 여전히 unsafe (포인터 연산) — 동시성 안전은 호출자 책임.

use core::cell::UnsafeCell;

#[repr(transparent)]
pub struct SyncUnsafeCell<T: ?Sized>(UnsafeCell<T>);

unsafe impl<T: ?Sized> Sync for SyncUnsafeCell<T> {}

impl<T> SyncUnsafeCell<T> {
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    /// 내부 값 가변 포인터.
    /// 호출자는 동시성 안전을 보장해야 함 (critical section 또는 단일 소유).
    #[inline]
    pub const fn get(&self) -> *mut T {
        self.0.get()
    }
}
