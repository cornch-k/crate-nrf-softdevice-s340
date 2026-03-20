// ANT+ event polling — integrates with Softdevice::run().
// C original: ant_evt_handler() registered via NRF_SDH_ANT_OBSERVER
//
// ANT events are separate from BLE events.
// BLE: sd_ble_evt_get()
// ANT: sd_ant_event_get()
// Both are woken by the same SWI2 interrupt.

use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;
use futures::future::poll_fn;

use crate::raw;

// Shares waker with BLE — SWI2 interrupt wakes both.
static ANT_EVT_WAKER: AtomicWaker = AtomicWaker::new();

/// ANT event data from sd_ant_event_get().
pub struct AntEvent {
    pub channel: u8,
    pub event: u8,
    pub msg: [u8; 41], // MESG_BUFFER_SIZE from ant_parameters.h
}

/// ANT event callback type.
pub type AntEventHandler = fn(event: &AntEvent);

/// Run the ANT event loop. Call from Softdevice::run().
/// Polls sd_ant_event_get() and dispatches to handler.
pub(crate) async fn run_ant<F: FnMut(&AntEvent)>(mut handler: F) -> ! {
    trace!("run_ant: started");
    loop {
        // Poll ANT events periodically instead of relying on SWI2 waker.
        // S340 may not trigger SWI2 for ANT events the same way as BLE.
        embassy_futures::yield_now().await;

        unsafe {
            let mut channel: u8 = 0;
            let mut event: u8 = 0;
            let mut msg = [0u8; 41];

            loop {
                let ret = raw::ant::data::sd_ant_event_get(
                    &mut channel,
                    &mut event,
                    msg.as_mut_ptr(),
                );

                if ret == 0 {
                    trace!("ant_evt: ch={} evt={}", channel, event);
                    let evt = AntEvent { channel, event, msg };
                    handler(&evt);
                } else {
                    break;
                }
            }
        }
    }
}

/// Wake the ANT event poller. Called from SWI2 interrupt handler.
pub(crate) fn wake() {
    ANT_EVT_WAKER.wake();
}
