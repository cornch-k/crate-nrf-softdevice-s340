//! Bluetooth Low Energy

mod connection;
mod gap;
mod gatt_traits;
mod replies;
mod types;

pub use connection::*;
pub use gap::*;
pub use gatt_traits::*;
pub use types::*;

mod common;

#[cfg(feature = "ble-sec")]
pub mod security;

#[cfg(feature = "ble-central")]
pub mod central;

#[cfg(feature = "ble-peripheral")]
pub mod advertisement_builder;
#[cfg(feature = "ble-peripheral")]
pub mod peripheral;

#[cfg(feature = "ble-gatt-client")]
pub mod gatt_client;

#[cfg(feature = "ble-gatt-server")]
pub mod gatt_server;

#[cfg(feature = "ble-l2cap")]
pub mod l2cap;

use core::mem;

#[cfg(any(feature = "ble-gatt-server", feature = "ble-sec"))]
pub use replies::*;

use crate::util::get_union_field;
use crate::{raw, RawError, Softdevice};

pub(crate) unsafe fn on_evt(ble_evt: *const raw::ble_evt_t) {
    trace!("ble evt {:?}", (*ble_evt).header.evt_id as u32);
    match (*ble_evt).header.evt_id as u32 {
        raw::BLE_EVT_BASE..=raw::BLE_EVT_LAST => common::on_evt(ble_evt),
        raw::BLE_GAP_EVT_BASE..=raw::BLE_GAP_EVT_LAST => gap::on_evt(ble_evt),
        #[cfg(feature = "ble-gatt-client")]
        raw::BLE_GATTC_EVT_BASE..=raw::BLE_GATTC_EVT_LAST => gatt_client::on_evt(ble_evt),
        #[cfg(feature = "ble-gatt-server")]
        raw::BLE_GATTS_EVT_BASE..=raw::BLE_GATTS_EVT_LAST => gatt_server::on_evt(ble_evt),
        #[cfg(feature = "ble-l2cap")]
        raw::BLE_L2CAP_EVT_BASE..=raw::BLE_L2CAP_EVT_LAST => l2cap::on_evt(ble_evt),
        // Central-only: reply to peripheral-initiated MTU exchange.
        // Without this, the pending exchange blocks sd_ble_gatts_hvx().
        #[cfg(not(feature = "ble-gatt-server"))]
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_EXCHANGE_MTU_REQUEST => {
            let gatts_evt = get_union_field(ble_evt, &(*ble_evt).evt.gatts_evt);
            let ret = raw::sd_ble_gatts_exchange_mtu_reply(
                gatts_evt.conn_handle,
                raw::BLE_GATT_ATT_MTU_DEFAULT as u16,
            );
            if let Err(_e) = RawError::convert(ret) {
                warn!("sd_ble_gatts_exchange_mtu_reply err {:?}", _e);
            }
        }

        // Central-only: respond to SYS_ATTR_MISSING so the SoftDevice
        // can proceed with GATTC HVX delivery.
        #[cfg(not(feature = "ble-gatt-server"))]
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_SYS_ATTR_MISSING => {
            let gatts_evt = get_union_field(ble_evt, &(*ble_evt).evt.gatts_evt);
            let ret = raw::sd_ble_gatts_sys_attr_set(
                gatts_evt.conn_handle,
                core::ptr::null(),
                0,
                0,
            );
            if let Err(_e) = RawError::convert(ret) {
                warn!("sd_ble_gatts_sys_attr_set err {:?}", _e);
            }
        }

        _ => {}
    }
}

pub fn get_address(_sd: &Softdevice) -> Address {
    unsafe {
        let mut addr: raw::ble_gap_addr_t = mem::zeroed();
        let ret = raw::sd_ble_gap_addr_get(&mut addr);
        unwrap!(RawError::convert(ret), "sd_ble_gap_addr_get");
        Address::from_raw(addr)
    }
}

pub fn set_address(_sd: &Softdevice, addr: &Address) {
    unsafe {
        let ret = raw::sd_ble_gap_addr_set(addr.as_raw());
        unwrap!(RawError::convert(ret), "sd_ble_gap_addr_set");
    }
}
