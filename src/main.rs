use bluest::AdvertisingDevice;
use eframe::egui;
use egui::CentralPanel;
use egui_inbox::UiInbox;

use flume;

use tokio::runtime::Runtime;

mod btnus;
use btnus::spawn_btnus_thread;

use crate::btnus::ThreadedNusMsg;

pub fn main() -> eframe::Result<()> {
    // let inbox: UiInbox<Option<String> = UiInbox::new();
    let inbox: UiInbox<Option<ThreadedNusMsg>> = UiInbox::new();
    let mut state: Option<ThreadedNusMsg> = None;
    let mut scan_vec: Vec<AdvertisingDevice> = vec![];

    let (cmd_tx, cmd_rx) = flume::unbounded();
    let tx = inbox.sender();
    spawn_btnus_thread(cmd_rx, tx);

    eframe::run_simple_native(
        "DnD Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                // `read` will return an iterator over all pending messages

                // loop through all received responses
                for response in inbox.read(ui) {
                    let m = response.clone();
                    // let m = response;
                    match m {
                        Some(ThreadedNusMsg::AmReadyIdle(_adapter_desc)) => {
                            state = Some(ThreadedNusMsg::AmReadyIdle("".into()));
                        }
                        Some(ThreadedNusMsg::AmNotReady)
                        | Some(ThreadedNusMsg::AmScanning)
                        | Some(ThreadedNusMsg::AmConnecting)
                        | Some(ThreadedNusMsg::AmConnected)
                        | Some(ThreadedNusMsg::AmDone) => {
                            state = m;
                        }
                        Some(ThreadedNusMsg::DataScanResult(new_scans)) => {
                            scan_vec.extend(new_scans);
                        }
                        Some(msg) => {
                            println!("TODO: handle msg = {msg:?}");
                        }
                        None => {
                            // ???
                        }
                    }
                }
                // if let Some(last) = inbox.read(ui).last() {
                //     state = last;
                // }
                // There also is a `replace` method that you can use as a shorthand for the above:
                // inbox.replace(ui, &mut state);

                if ui.button("Send First Command...Async Task").clicked() {
                    // state = Some("Waiting for async task to complete".to_string());
                    // let tx = inbox.sender();
                    // spawn_btnus_thread(tx);

                    let send_res = cmd_tx.send(Some(ThreadedNusMsg::AmReadyIdle("".into())));
                    println!("send_res = {send_res:?}");
                }

                ui.label(format!("State: {:?}", state));
                ui.label(format!("Found {} devices", scan_vec.len()));
                ui.label(format!("Devices = {:?}", scan_vec));
            });
        },
    )
}
