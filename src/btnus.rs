// use std::thread;

// use std::time::Duration;

use bluest::{Adapter, AdvertisingDevice};
use flume::Receiver;

use futures_lite::StreamExt;

// use flume::async::RecvStream;
use tokio::runtime::Runtime;
use tokio::time::{Duration, timeout};
use tracing::{debug, error, info, trace, warn};
// use tracing::{error, info, warn};

use egui_inbox::UiInboxSender;

#[derive(Debug, Clone, PartialEq)]
pub enum ThreadedNusMsg {
    // Commands
    // Ready,
    // StartConnect(Vec<u8>),
    DoScanStart(String), // FIXME: put scan params as a type in this event
    DoScanStop,
    DoConnect(AdvertisingDevice),
    DoDisconnect,
    DoQuit,
    //
    DataScanResult(Vec<AdvertisingDevice>),
    DataTx(Vec<u8>),
    DataRx(Vec<u8>),
    //
    AmNotReady,
    AmReadyIdle(String),
    AmScanning,
    AmConnecting,
    AmConnected,
    AmDone,
}
use ThreadedNusMsg::*;

pub fn spawn_btnus_thread(
    cmd: flume::Receiver<ThreadedNusMsg>,
    resp: egui_inbox::UiInboxSender<ThreadedNusMsg>,
) {
    std::thread::spawn(move || {
        let rt = Runtime::new().expect("Failed to create runtime");
        rt.block_on(async {
            loop {
                // TODO: put this in an async function that returns result and use ? operator???
                let adapter = Adapter::default().await;
                if adapter.is_none() {
                    resp.send(AmNotReady).ok();
                    std::thread::sleep(Duration::from_millis(1000));
                    continue;
                }
                let adapter = adapter.unwrap(); // simplify below code
                let _ = adapter.wait_available().await;
                resp.send(AmReadyIdle(format!("{:?}", &adapter))).ok();

                info!("btnus waiting for {:?}", DoScanStart("".into()));
                loop {
                    match cmd.recv_async().await {
                        Ok(DoScanStart(_opts)) => {
                            break;
                        }
                        // TODO: handle connect device
                        Ok(unhandled) => {
                            warn!("unhandled message waiting for DoScanStart(_) = {unhandled:?}");
                        }
                        Err(_bad) => {
                            //
                        }
                    }
                }

                info!("starting scan");
                let mut scan = adapter.scan(&[]).await;
                if scan.is_err() {
                    resp.send(AmNotReady).ok();
                    std::thread::sleep(Duration::from_millis(1000));
                    continue;
                }
                let mut scan = scan.unwrap();
                resp.send(AmScanning).ok();

                // if scan.is
                // match
                info!("scan started");
                while let Some(discovered_device) = scan.next().await {
                    // TODO: put this timeout recv in a helper for readability
                    // TODO: check if the sync method recv_timeout works just fine in here... it
                    // should...
                    match cmd.recv_timeout(Duration::from_millis(10)) {
                        Ok(DoScanStop) => {
                            info!("scan: recv'd DoScanStop, stopping scan");
                            break;
                        }
                        // TODO: handle connect
                        Ok(unhandled) => {
                            warn!("scan: unhandled = {unhandled:?}");
                        }
                        Err(to) => {
                            trace!("timeout waiting for msg during scan: {to}");
                            //
                        }
                    }

                    // Wrap the future with a timeout of 1 second
                    resp.send(DataScanResult(vec![discovered_device.clone()]))
                        .ok();
                    info!(
                        "{}{}: {:?}",
                        discovered_device
                            .device
                            .name()
                            .as_deref()
                            .unwrap_or("(unknown)"),
                        discovered_device
                            .rssi
                            .map(|x| format!(" ({}dBm)", x))
                            .unwrap_or_default(),
                        discovered_device.adv_data.services
                    );
                }
                info!("scan stopped")
            } // outer forever loop
        });
    });
}
