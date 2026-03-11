use std::collections::HashMap;

use bluest::AdvertisingDevice;
use bluest::DeviceId;
use eframe::egui;
use egui::CentralPanel;
use egui_inbox::UiInbox;

use flume;

mod btnus;
use btnus::ThreadedNusMsg::*;
use btnus::spawn_btnus_thread;

use tracing::metadata::LevelFilter;
use tracing_subscriber::filter;
use tracing_subscriber::prelude::*;

use crate::btnus::ThreadedNusMsg;

pub fn main() -> eframe::Result<()> {
    // NOTE: logging/tracing config first
    let filter = filter::Targets::new()
        // Enable the `INFO` level for anything in `my_crate`
        .with_default(LevelFilter::INFO)
        .with_target("hope", LevelFilter::INFO)
        .with_target("bluest", LevelFilter::WARN);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    // NOTE: async/thread comms
    let inbox: UiInbox<ThreadedNusMsg> = UiInbox::new();
    let mut bt_state: ThreadedNusMsg = AmNotReady;
    let mut scan_vec: Vec<AdvertisingDevice> = vec![];
    let mut scan_map: HashMap<DeviceId, AdvertisingDevice> = HashMap::default();

    let (cmd_tx, cmd_rx) = flume::unbounded();
    let resp_tx = inbox.sender();

    // NOTE: spawn btnus thread with async runtime
    spawn_btnus_thread(cmd_rx, resp_tx);

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
                        AmReadyIdle(adapter_desc) => {
                            bt_state = AmReadyIdle(adapter_desc.clone());
                        }
                        AmNotReady | AmScanning | AmConnecting | AmConnected | AmDone => {
                            bt_state = m;
                        }
                        DataScanResult(recvd_scans) => {
                            // scan_vec.extend(new_scans);
                            for adv_dev in recvd_scans {
                                let id = adv_dev.device.id();
                                let unique = !scan_map.contains_key(&id);
                                if unique {
                                    scan_map.insert(id, adv_dev);
                                }
                            }
                        }
                        msg => {
                            println!("TODO: handle msg = {msg:?}");
                        }
                    }
                }

                ui.label(format!("State: {:?}", bt_state));
                ui.label(format!("Found {} devices", scan_map.len()));

                // scan start/stop
                ui.horizontal(|ui| {
                    let start_enabled = matches!(bt_state, AmReadyIdle(_));
                    let stop_enabled = matches!(bt_state, AmScanning);
                    ui.add_enabled_ui(start_enabled, |ui| {
                        if ui.button("Start Scan").clicked() {
                            // TODO: provide actual scan options into DoScanStart message from UI
                            let send_start_scan_res = cmd_tx.send(DoScanStart("".into()));
                            println!("send_start_scan_res = {send_start_scan_res:?}");
                        }
                    });
                    ui.add_enabled_ui(stop_enabled, |ui| {
                        if ui.button("Stop Scan").clicked() {
                            let send_stop_scan_res = cmd_tx.send(DoScanStop);
                            println!("send_stop_scan_res = {send_stop_scan_res:?}");
                        }
                    });
                });
            });
        },
    )
}
