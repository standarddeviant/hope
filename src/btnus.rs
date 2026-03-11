// use std::thread;

use std::time::Duration;

use bluest::{Adapter, AdvertisingDevice};
use flume::Receiver;

use futures_lite::StreamExt;

// use flume::async::RecvStream;
use tokio::runtime::Runtime;
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

pub fn spawn_btnus_thread(
    cmd: flume::Receiver<Option<ThreadedNusMsg>>,
    resp: egui_inbox::UiInboxSender<Option<ThreadedNusMsg>>,
) {
    std::thread::spawn(move || {
        let rt = Runtime::new().expect("Failed to create runtime");
        rt.block_on(async {
            // std::thread::sleep(std::time::Duration::from_secs(1));
            // Send will return an error if the receiver has been dropped
            // but unless you have a long running task that will send multiple messages
            // you can just ignore the error

            println!("btnus waiting...");
            let first_cmd = cmd.recv_async().await;
            println!("btnus got {first_cmd:?} (yay!)");

            loop {
                // TODO: put this in an async function that returns result and use ? operator???
                let adapter = Adapter::default().await;
                if adapter.is_none() {
                    resp.send(Some(ThreadedNusMsg::AmNotReady)).ok();
                    std::thread::sleep(Duration::from_millis(1000));
                    continue;
                }
                let adapter = adapter.unwrap(); // simplify below code

                let msg = ThreadedNusMsg::AmReadyIdle(format!("{:?}", &adapter));
                // Hello from another thread!".to_string()))
                resp.send(Some(msg)).ok();

                println!("starting scan");
                let mut scan = adapter.scan(&[]).await;
                if scan.is_err() {
                    resp.send(Some(ThreadedNusMsg::AmNotReady)).ok();
                }
                let mut scan = scan.unwrap();

                // if scan.is
                // match
                println!("scan started");
                while let Some(discovered_device) = scan.next().await {
                    resp.send(Some(ThreadedNusMsg::DataScanResult(vec![
                        discovered_device.clone(),
                    ])))
                    .ok();
                    println!(
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
            }

            // TODO: use adapter...
            // ada
        });
    });
}

// pub fn spawn_bt_thread(
//     // cmd_recv: Receiver<ThreadedNusMsg>,
//     resp_send: egui_inbox::UiInboxSender<ThreadedNusMsg>,
// ) -> JoinHandle<u32> {
//     // TODO: move this out of new for clean
//
//     let handle = thread::spawn(move || {
//         let mut rc: u32 = 0;
//
//         // get bt-adapter and embed it in Context
//
//         let rt = Runtime::new().expect("Failed to create runtime");
//
//         // 3. Block the *current* new thread to run the async task to completion
//         rt.block_on(async {
//             // my_async_task().await;
//
//             let ble_adapter = bluest::Adapter::default()
//                 .await
//                 .expect("Can't obtain BLE adapter");
//
//             loop {
//                 match cmd_recv.recv() {
//                     Ok(ThreadedNusMsg::AmReadyIdle) => {
//                         break;
//                         //
//                     }
//                     Ok(msg) => {
//                         warn!("unexpected cmd while waiting for AmReadyIdle: {msg:?}");
//                     }
//                     Err(e) => {
//                         error!("cmd_recv recv failed: {e:?}");
//                         // break;
//                     }
//                 }
//             }
//
//             info!("sending AmReadyIdle");
//             match resp_send.send(ThreadedNusMsg::AmReadyIdle) {
//                 Ok(_good) => info!("AmReadyIdle sent"),
//                 Err(e) => error!("AmReadyIdle send failed: {e:?}"),
//             }
//
//             // put loop here
//         }); // end rt.block_on
//         rc
//     });
//
//     return handle;
// }
