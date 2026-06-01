use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m::peripheral::NVIC;

use crate::{raw, Interrupt, RawError, SocEvent};

unsafe extern "C" fn fault_handler(id: u32, pc: u32, info: u32) {
    // 2026-05-13 fork patch: SD-side pc 만으론 user 코드 위치 식별 불가.
    // hardware exception frame (SP 가 가리키는 [R0,R1,R2,R3,R12,LR,PC,xPSR])
    // 안의 user PC/LR 도 dump 해서 콘치님이 addr2line 으로 매핑 가능하도록.
    //
    // ARM Cortex-M exception entry 시 stack push frame layout (low→high addr):
    //   [+0]R0 [+4]R1 [+8]R2 [+12]R3 [+16]R12 [+20]LR [+24]PC [+28]xPSR
    //
    // fault_handler 가 SD callback chain 안 — SP 가 SD local stack 깊이 가리킴.
    // user exception frame 은 그 위쪽 (higher address) 어딘가. 첫 32 word dump
    // 해서 콘치님이 code 영역 (0x31000~0xAFFFF) 안 값 직접 식별.
    //
    // 저장 채널: (1) RTT 발사 (cargo run live 시) (2) flash 0xB0000 (RTT 끊김 대비)
    // NVMC direct write — cortex_m::interrupt::disable 안 함 (SD timing 보호).
    // SD 가 이미 fault 상태라 NVMC 충돌 위험 낮음.
    let msp: u32;
    let psp: u32;
    unsafe {
        core::arch::asm!("mrs {}, msp", out(reg) msp, options(nomem, nostack, preserves_flags));
        core::arch::asm!("mrs {}, psp", out(reg) psp, options(nomem, nostack, preserves_flags));
    }

    let mut dump = [0u32; 32];
    for i in 0..32usize {
        unsafe {
            dump[i] = core::ptr::read_volatile((msp + (i as u32) * 4) as *const u32);
        }
    }

    // ── flash 0xB0000 NVMC direct write ──────────────────────────
    // Layout: [magic, id, sd_pc, info, msp, psp, dump[0..32]]  총 38 words
    const PANIC_FLASH_ADDR: u32 = 0xB_0000;
    const PANIC_FLASH_MAGIC: u32 = 0xDEAD_BEEF;
    const NVMC_READY: *const u32 = 0x4001_E400 as *const u32;
    const NVMC_CONFIG: *mut u32 = 0x4001_E504 as *mut u32;
    const NVMC_ERASEPAGE: *mut u32 = 0x4001_E508 as *mut u32;

    unsafe {
        while core::ptr::read_volatile(NVMC_READY) == 0 {}
        core::ptr::write_volatile(NVMC_CONFIG, 2); // EEN
        while core::ptr::read_volatile(NVMC_READY) == 0 {}
        core::ptr::write_volatile(NVMC_ERASEPAGE, PANIC_FLASH_ADDR);
        while core::ptr::read_volatile(NVMC_READY) == 0 {}
        core::ptr::write_volatile(NVMC_CONFIG, 1); // WEN
        while core::ptr::read_volatile(NVMC_READY) == 0 {}

        let header: [u32; 6] = [PANIC_FLASH_MAGIC, id, pc, info, msp, psp];
        for (i, word) in header.iter().enumerate() {
            let addr = PANIC_FLASH_ADDR + (i as u32) * 4;
            core::ptr::write_volatile(addr as *mut u32, *word);
            while core::ptr::read_volatile(NVMC_READY) == 0 {}
        }
        for (i, word) in dump.iter().enumerate() {
            let addr = PANIC_FLASH_ADDR + ((6 + i) as u32) * 4;
            core::ptr::write_volatile(addr as *mut u32, *word);
            while core::ptr::read_volatile(NVMC_READY) == 0 {}
        }

        core::ptr::write_volatile(NVMC_CONFIG, 0); // REN
        while core::ptr::read_volatile(NVMC_READY) == 0 {}
    }

    // RTT 발사 — flash write 실패 시 fallback (RTT live 면 dump 직접 받음)
    defmt::error!(
        "FAULT_HANDLER | sd_pc=0x{:08x} info=0x{:08x} msp=0x{:08x} psp=0x{:08x} (flash log @ 0xB0000)",
        pc, info, msp, psp
    );
    defmt::error!(
        "MSP+00..15: {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x}",
        dump[0], dump[1], dump[2], dump[3], dump[4], dump[5], dump[6], dump[7],
        dump[8], dump[9], dump[10], dump[11], dump[12], dump[13], dump[14], dump[15]
    );
    defmt::error!(
        "MSP+16..31: {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x}",
        dump[16], dump[17], dump[18], dump[19], dump[20], dump[21], dump[22], dump[23],
        dump[24], dump[25], dump[26], dump[27], dump[28], dump[29], dump[30], dump[31]
    );

    match (id, info) {
        (raw::NRF_FAULT_ID_SD_ASSERT, _) => panic!(
            "Softdevice assertion failed: an assertion inside the softdevice's code has failed. Most common cause is disabling interrupts for too long. Make sure you're using nrf_softdevice::interrupt::free instead of cortex_m::interrupt::free, which disables non-softdevice interrupts only. PC={:x}",
            pc
        ),
        (raw::NRF_FAULT_ID_APP_MEMACC, 0) => panic!(
            "Softdevice memory access violation. Your program accessed RAM reserved to the softdevice. PC={:x}",
            pc
        ),
        (raw::NRF_FAULT_ID_APP_MEMACC, _) => panic!(
            "Softdevice memory access violation. Your program accessed registers for a peripheral reserved to the softdevice. PC={:x} PREGION={:?}",
            pc, info
        ),
        _ => panic!(
            "Softdevice unknown fault id={:?} pc={:x} info={:?}",
            id, pc, info
        ),
    }
}

/// Singleton instance of the enabled softdevice.
///
/// The `Softdevice` instance can be obtaind by enabling it with [`Softdevice::enable`]. Once
/// enabled, it can be used to establish Bluetooth connections with [`ble::central`] and [`ble::peripheral`].
///
/// Disabling the softdevice is not supported due to the complexity of a safe implementation. Consider resetting the CPU instead.
pub struct Softdevice {
    // Prevent Send, Sync
    _private: PhantomData<*mut ()>,
    #[cfg(feature = "ble-gatt")]
    #[allow(unused)]
    pub(crate) att_mtu: u16,
    #[cfg(feature = "ble-l2cap")]
    pub(crate) l2cap_rx_mps: u16,
}

/// Softdevice configuration.
///
/// Fields set to None will use a default configuration.
#[derive(Default)]
pub struct Config {
    pub clock: Option<raw::nrf_clock_lf_cfg_t>,
    pub conn_gap: Option<raw::ble_gap_conn_cfg_t>,
    pub conn_gattc: Option<raw::ble_gattc_conn_cfg_t>,
    pub conn_gatts: Option<raw::ble_gatts_conn_cfg_t>,
    pub conn_gatt: Option<raw::ble_gatt_conn_cfg_t>,
    #[cfg(feature = "ble-l2cap")]
    pub conn_l2cap: Option<raw::ble_l2cap_conn_cfg_t>,
    pub common_vs_uuid: Option<raw::ble_common_cfg_vs_uuid_t>,
    pub gap_role_count: Option<raw::ble_gap_cfg_role_count_t>,
    pub gap_device_name: Option<raw::ble_gap_cfg_device_name_t>,
    #[cfg(not(feature = "s340"))]
    pub gap_ppcp_incl: Option<raw::ble_gap_cfg_ppcp_incl_cfg_t>,
    #[cfg(not(feature = "s340"))]
    pub gap_car_incl: Option<raw::ble_gap_cfg_car_incl_cfg_t>,
    pub gatts_service_changed: Option<raw::ble_gatts_cfg_service_changed_t>,
    pub gatts_attr_tab_size: Option<raw::ble_gatts_cfg_attr_tab_size_t>,
}

const APP_CONN_CFG_TAG: u8 = 1;

fn get_app_ram_base() -> u32 {
    // flip-link 가 cortex-m-rt 의 `_ram_start` 와 우리가 PROVIDE 한 모든
    // RAM 영역 symbol 을 stack 끝 (= __sdata) 으로 override 하기 때문에 link
    // symbol 로는 진짜 RAM ORIGIN 을 못 가져옴. 이 fork 는 nrf52840 + S340 +
    // flip-link 환경 전용이라 등록된 SoftDevice 가 사용하는 RAM 영역 끝 주소를 하드코딩.
    //
    // !! memory.x 의 RAM ORIGIN 변경 시 여기도 동기화 필수 !!
    // 2026-05-19: vs_uuid_count 5 → 9 위해 +0x800 (2048 byte). memory.x 와 동기.
    // 2026-05-20: APP_MEMACC fault 발생 — 동작 중 SD RAM 침범. +0x1000 추가.
    // 2026-05-20: +0x1000 도 부족, 동일 fault. +0x4000 은 .bss overflow.
    // +0x2000 (8KB) 추가하여 0x2000B788.
    0x2000B788
}

fn cfg_set(id: u32, cfg: &raw::ble_cfg_t) {
    let app_ram_base = get_app_ram_base();
    let ret = unsafe { raw::sd_ble_cfg_set(id, cfg, app_ram_base) };
    match RawError::convert(ret) {
        Ok(()) => {}
        Err(RawError::NoMem) => {}
        Err(err) => panic!("sd_ble_cfg_set {:?} err {:?}", id, err),
    }
}

static ENABLED: AtomicBool = AtomicBool::new(false);

/// System OFF 진입. 이 함수는 리턴하지 않음.
///
/// SoftDevice를 통해 POWER->SYSTEMOFF 레지스터를 설정.
/// GPIO SENSE가 설정된 핀으로 wake 가능 (시스템 리셋).
pub fn system_off() -> ! {
    unsafe { raw::sd_power_system_off(); }
    loop { cortex_m::asm::wfi(); }
}

/// BLE GAP Preferred Connection Parameters 설정.
pub fn set_ppcp(params: &raw::ble_gap_conn_params_t) {
    let ret = unsafe { raw::sd_ble_gap_ppcp_set(params) };
    assert!(ret == raw::NRF_SUCCESS, "sd_ble_gap_ppcp_set failed: {}", ret);
}

/// BLE GAP 연결 파라미터 업데이트 요청.
pub fn conn_param_update(conn_handle: u16, params: &raw::ble_gap_conn_params_t) -> u32 {
    unsafe { raw::sd_ble_gap_conn_param_update(conn_handle, params) }
}

/// POWER->RESETREAS 읽고 클리어. 부팅 직후 호출.
pub fn reset_reason_take() -> u32 {
    let mut reason: u32 = 0;
    unsafe {
        raw::sd_power_reset_reason_get(&mut reason);
        raw::sd_power_reset_reason_clr(0xFFFF_FFFF);
    }
    reason
}

/// 읽기 전용 GAP 보안 모드 (no access).
/// device name 같이 read-only 속성에 사용.
pub fn sec_mode_no_access() -> raw::ble_gap_conn_sec_mode_t {
    raw::ble_gap_conn_sec_mode_t {
        _bitfield_align_1: [],
        _bitfield_1: raw::__BindgenBitfieldUnit::new([0u8; 1]),
    }
}

/// GATTS sys_attrs 읽기. 성공 시 0, 실패 시 에러 코드 반환.
pub fn raw_sys_attr_get(conn_handle: u16, buf: &mut [u8], len: &mut u16) -> u32 {
    unsafe { raw::sd_ble_gatts_sys_attr_get(conn_handle, buf.as_mut_ptr(), len, 0) }
}
// `static mut` 회피: SyncUnsafeCell로 래핑. ENABLED AtomicBool이 초기화 완료 여부를 보장.
use crate::util::SyncUnsafeCell;
static SOFTDEVICE: SyncUnsafeCell<MaybeUninit<Softdevice>> =
    SyncUnsafeCell::new(MaybeUninit::uninit());

impl Softdevice {
    /// Enable the softdevice.
    ///
    /// # Panics
    /// - Panics if the requested configuration requires more memory than reserved for the softdevice. In that case, you can give more memory to the softdevice by editing the RAM start address in `memory.x`. The required start address is logged prior to panic.
    /// - Panics if the requested configuration has too high memory requirements for the softdevice. The softdevice supports a maximum dynamic memory size of 64kb.
    /// - Panics if called multiple times. Must be called at most once.
    pub fn enable(config: &Config) -> &'static mut Softdevice {
        if ENABLED
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            panic!("nrf_softdevice::enable() called multiple times.")
        }

        let p_clock_lf_cfg = config.clock.as_ref().map(|x| x as _).unwrap_or(ptr::null());
        #[cfg(feature = "s340")]
        let ret = unsafe {
            // S340 requires ANT license key as 3rd parameter.
            const ANT_LICENSE_KEY: &[u8] = b"49a5-b6af-88da-b010-fb67-aee8-0778-9f58\0";
            raw::sd_softdevice_enable(
                p_clock_lf_cfg,
                Some(fault_handler),
                ANT_LICENSE_KEY.as_ptr() as *const _,
            )
        };
        #[cfg(not(feature = "s340"))]
        let ret = unsafe { raw::sd_softdevice_enable(p_clock_lf_cfg, Some(fault_handler)) };
        match RawError::convert(ret) {
            Ok(()) => {}
            Err(err) => panic!("sd_softdevice_enable err {:?}", err),
        }

        let app_ram_base = get_app_ram_base();

        // Set at least one GAP config so conn_cfg_tag 1 (APP_CONN_CFG_TAG) is usable.
        // If you set none, it seems the softdevice won't let you use it, requiring a conn_cfg_tag of 0 (raw::BLE_CONN_CFG_TAG_DEFAULT) instead.
        let val = config.conn_gap.unwrap_or(raw::ble_gap_conn_cfg_t {
            conn_count: raw::BLE_GAP_CONN_COUNT_DEFAULT as u8,
            event_length: raw::BLE_GAP_EVENT_LENGTH_DEFAULT as u16,
        });
        cfg_set(
            raw::BLE_CONN_CFGS_BLE_CONN_CFG_GAP,
            &raw::ble_cfg_t {
                conn_cfg: raw::ble_conn_cfg_t {
                    conn_cfg_tag: APP_CONN_CFG_TAG,
                    params: raw::ble_conn_cfg_t__bindgen_ty_1 { gap_conn_cfg: val },
                },
            },
        );

        if let Some(val) = config.conn_gatt {
            cfg_set(
                raw::BLE_CONN_CFGS_BLE_CONN_CFG_GATT,
                &raw::ble_cfg_t {
                    conn_cfg: raw::ble_conn_cfg_t {
                        conn_cfg_tag: APP_CONN_CFG_TAG,
                        params: raw::ble_conn_cfg_t__bindgen_ty_1 { gatt_conn_cfg: val },
                    },
                },
            );
        }

        if let Some(val) = config.conn_gattc {
            cfg_set(
                raw::BLE_CONN_CFGS_BLE_CONN_CFG_GATTC,
                &raw::ble_cfg_t {
                    conn_cfg: raw::ble_conn_cfg_t {
                        conn_cfg_tag: APP_CONN_CFG_TAG,
                        params: raw::ble_conn_cfg_t__bindgen_ty_1 { gattc_conn_cfg: val },
                    },
                },
            );
        }

        if let Some(val) = config.conn_gatts {
            cfg_set(
                raw::BLE_CONN_CFGS_BLE_CONN_CFG_GATTS,
                &raw::ble_cfg_t {
                    conn_cfg: raw::ble_conn_cfg_t {
                        conn_cfg_tag: APP_CONN_CFG_TAG,
                        params: raw::ble_conn_cfg_t__bindgen_ty_1 { gatts_conn_cfg: val },
                    },
                },
            );
        }

        #[cfg(feature = "ble-l2cap")]
        if let Some(val) = config.conn_l2cap {
            cfg_set(
                raw::BLE_CONN_CFGS_BLE_CONN_CFG_L2CAP,
                &raw::ble_cfg_t {
                    conn_cfg: raw::ble_conn_cfg_t {
                        conn_cfg_tag: APP_CONN_CFG_TAG,
                        params: raw::ble_conn_cfg_t__bindgen_ty_1 { l2cap_conn_cfg: val },
                    },
                },
            );
        }

        if let Some(val) = config.common_vs_uuid {
            cfg_set(
                raw::BLE_COMMON_CFGS_BLE_COMMON_CFG_VS_UUID,
                &raw::ble_cfg_t {
                    common_cfg: raw::ble_common_cfg_t { vs_uuid_cfg: val },
                },
            );
        }

        if let Some(val) = config.gap_role_count {
            cfg_set(
                raw::BLE_GAP_CFGS_BLE_GAP_CFG_ROLE_COUNT,
                &raw::ble_cfg_t {
                    gap_cfg: raw::ble_gap_cfg_t { role_count_cfg: val },
                },
            );
        }

        if let Some(val) = config.gap_device_name {
            cfg_set(
                raw::BLE_GAP_CFGS_BLE_GAP_CFG_DEVICE_NAME,
                &raw::ble_cfg_t {
                    gap_cfg: raw::ble_gap_cfg_t { device_name_cfg: val },
                },
            );
        }

        #[cfg(not(feature = "s340"))]
        if let Some(val) = config.gap_ppcp_incl {
            cfg_set(
                raw::BLE_GAP_CFGS_BLE_GAP_CFG_PPCP_INCL_CONFIG,
                &raw::ble_cfg_t {
                    gap_cfg: raw::ble_gap_cfg_t { ppcp_include_cfg: val },
                },
            );
        }

        #[cfg(not(feature = "s340"))]
        if let Some(val) = config.gap_car_incl {
            cfg_set(
                raw::BLE_GAP_CFGS_BLE_GAP_CFG_CAR_INCL_CONFIG,
                &raw::ble_cfg_t {
                    gap_cfg: raw::ble_gap_cfg_t { car_include_cfg: val },
                },
            );
        }
        if let Some(val) = config.gatts_service_changed {
            cfg_set(
                raw::BLE_GATTS_CFGS_BLE_GATTS_CFG_SERVICE_CHANGED,
                &raw::ble_cfg_t {
                    gatts_cfg: raw::ble_gatts_cfg_t { service_changed: val },
                },
            );
        }
        if let Some(val) = config.gatts_attr_tab_size {
            cfg_set(
                raw::BLE_GATTS_CFGS_BLE_GATTS_CFG_ATTR_TAB_SIZE,
                &raw::ble_cfg_t {
                    gatts_cfg: raw::ble_gatts_cfg_t { attr_tab_size: val },
                },
            );
        }

        let mut wanted_app_ram_base = app_ram_base;
        let ret = unsafe { raw::sd_ble_enable(&mut wanted_app_ram_base as _) };
        info!("softdevice RAM: {:?} bytes", wanted_app_ram_base - 0x20000000);
        match RawError::convert(ret) {
            Ok(()) => {}
            Err(RawError::NoMem) => {
                if wanted_app_ram_base <= app_ram_base {
                    panic!("selected configuration has too high RAM requirements.")
                } else {
                    panic!(
                        "too little RAM for softdevice. Change your app's RAM start address to {:x}",
                        wanted_app_ram_base
                    );
                }
            }
            Err(err) => panic!("sd_ble_enable err {:?}", err),
        }

        if wanted_app_ram_base < app_ram_base {
            warn!("You're giving more RAM to the softdevice than needed. You can change your app's RAM start address to {:x}", wanted_app_ram_base);
        }

        unsafe {
            NVIC::unmask(Interrupt::SWI2_EGU2);
        }

        #[cfg(feature = "ble-gatt")]
        let att_mtu = config
            .conn_gatt
            .map(|x| x.att_mtu)
            .unwrap_or(raw::BLE_GATT_ATT_MTU_DEFAULT as u16);

        #[cfg(feature = "ble-l2cap")]
        let l2cap_rx_mps = config
            .conn_l2cap
            .map(|x| x.rx_mps)
            .unwrap_or(raw::BLE_L2CAP_MPS_MIN as u16);

        let sd = Softdevice {
            _private: PhantomData,

            #[cfg(feature = "ble-gatt")]
            att_mtu,

            #[cfg(feature = "ble-l2cap")]
            l2cap_rx_mps,
        };

        // SAFETY: ENABLED가 처음 false→true로 바뀐 이 경로만 여기 진입 가능 (compare_exchange 위).
        // 다른 경로는 panic 또는 ENABLED 체크로 진입 못함.
        unsafe {
            let p = (*SOFTDEVICE.get()).as_mut_ptr();
            p.write(sd);
            &mut *p
        }
    }

    /// Return an instance to the softdevice without checking whether
    /// it is enabled or not. This is only safe if the softdevice is enabled
    /// (a call to [`enable`] has returned without error) and no `&mut` references
    /// to the softdevice are active
    pub unsafe fn steal() -> &'static Softdevice {
        // SAFETY: caller가 enable() 선행 호출을 보장하고 &mut 참조 활성화 상태가 없어야 함.
        &*(*SOFTDEVICE.get()).as_ptr()
    }

    /// Runs the softdevice event handling loop.
    ///
    /// It must be called in its own async task after enabling the softdevice
    /// and before doing any operation. Failure to doing so will cause async operations to never finish.
    pub async fn run(&self) -> ! {
        self.run_with_callback(|_| ()).await
    }

    /// Runs the softdevice event handling loop with a callback for [`SocEvent`]s.
    ///
    /// It must be called under the same conditions as [`Softdevice::run()`]. This
    /// version allows the application to provide a callback to receive SoC events
    /// from the softdevice (other than flash events which are handled by [`Flash`](crate::flash::Flash)).
    pub async fn run_with_callback<F: FnMut(SocEvent)>(&self, f: F) -> ! {
        #[cfg(feature = "s340")]
        {
            embassy_futures::join::join3(
                self.run_ble(),
                crate::events::run_soc(f),
                crate::ant::event::run_ant(|evt| {
                    crate::ant::dispatch_event(evt);
                }),
            ).await;
        }
        #[cfg(not(feature = "s340"))]
        {
            embassy_futures::join::join(self.run_ble(), crate::events::run_soc(f)).await;
        }
        // Should never get here
        loop {}
    }

    /// Runs the softdevice soc event handler only.
    ///
    /// It must be called under the same conditions as [`Softdevice::run()`].
    pub async fn run_soc(&self) -> ! {
        crate::events::run_soc(|_| ()).await
    }

    /// Runs the softdevice ble event handler only.
    ///
    /// It must be called under the same conditions as [`Softdevice::run()`].
    pub async fn run_ble(&self) -> ! {
        crate::events::run_ble().await
    }
}
