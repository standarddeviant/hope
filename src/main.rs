use std::collections::HashMap;

use bluest::AdvertisingDevice;
use bluest::DeviceId;
use eframe::{App, CreationContext, Frame, egui};
use egui::{Align, CentralPanel, Context, Layout, ThemePreference};
use egui_extras::Column;
use egui_inbox::{UiInbox, UiInboxSender};
use egui_selectable_table::SelectableTable;

use flume;

mod btnus;
mod scan_table;

use btnus::ThreadedNusMsg::*;
use btnus::spawn_btnus_thread;

use tracing::metadata::LevelFilter;
use tracing_subscriber::filter;
use tracing_subscriber::prelude::*;

use tracing::{info, warn};

use crate::btnus::ThreadedNusMsg;
use crate::scan_table::{ScanColumns, ScanConfig, ScanRow};

use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter}; // 0.25
//
use flume::Sender;

struct NusGui {
    // NOTE: async/thread comms
    cmd_tx: Sender<ThreadedNusMsg>,
    inbox: UiInbox<ThreadedNusMsg>, // = UiInbox::new();
    // resp_tx: UiInboxSender<ThreadedNusMsg>,
    bt_state: ThreadedNusMsg,                       // = AmNotReady;
    scan_vec: Vec<AdvertisingDevice>,               // = vec![];
    scan_map: HashMap<DeviceId, AdvertisingDevice>, // = HashMap::default();
    // scan_columns: Vec<ScanColumns>,                 //::iter().collect();

    // Auto reload after each 10k table row add or modification
    table: SelectableTable<ScanRow, ScanColumns, ScanConfig>,
}

impl NusGui {
    pub fn new(cc: &CreationContext) -> Self {
        cc.egui_ctx
            .options_mut(|a| a.theme_preference = ThemePreference::System);

        // NOTE: async/thread comms
        let inbox: UiInbox<ThreadedNusMsg> = UiInbox::new();
        let mut bt_state: ThreadedNusMsg = AmNotReady;
        let mut scan_vec: Vec<AdvertisingDevice> = vec![];
        let mut scan_map: HashMap<DeviceId, AdvertisingDevice> = HashMap::default();

        let scan_columns = ScanColumns::iter().collect();

        // Auto reload after each 10k table row add or modification
        let mut table = SelectableTable::new(scan_columns)
            .auto_reload(10_000)
            .auto_scroll()
            .horizontal_scroll()
            .select_full_row()
            .no_ctrl_a_capture();

        let (cmd_tx, cmd_rx) = flume::unbounded();
        let resp_tx = inbox.sender();

        // NOTE: spawn btnus thread with async runtime
        spawn_btnus_thread(cmd_rx, resp_tx);

        Self {
            cmd_tx,
            inbox,
            bt_state,
            scan_vec,
            scan_map,
            // scan_columns,
            table,
        }
    }
}

impl App for NusGui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            // `read` will return an iterator over all pending messages

            // loop through all received responses
            for response in self.inbox.read(ui) {
                let m = response.clone();
                // let m = response;
                match m {
                    AmReadyIdle(adapter_desc) => {
                        self.bt_state = AmReadyIdle(adapter_desc.clone());
                    }
                    AmNotReady | AmScanning | AmConnecting | AmConnected | AmDone => {
                        self.bt_state = m;
                    }
                    DataTx(nus_tx_bytes) => {
                        warn!("Unhandled NUS Tx Bytes = {nus_tx_bytes:?}");
                    }
                    DataScanResult(recvd_scans) => {
                        // scan_vec.extend(new_scans);
                        for adv_dev in recvd_scans {
                            let id = adv_dev.device.id();
                            let unique = !self.scan_map.contains_key(&id);
                            if unique {
                                self.scan_map.insert(id, adv_dev.clone());
                                self.scan_vec.push(adv_dev);
                                for scan_obj in &self.scan_vec {
                                    self.table.add_modify_row(|rows| {
                                        // edit row here
                                        for r in rows {
                                            if r.1.row_data.bt_id.eq(&Some(scan_obj.device.id())) {
                                                // copy the just-received thread_infos the correct table row correct
                                                // table row data
                                                r.1.row_data = scan_obj_to_scan_row(&scan_obj);
                                                return None; // indicate we modified a row, don't add a new one
                                            }
                                        }
                                        let scan_row = scan_obj_to_scan_row(&scan_obj);
                                        // indicate we didn't find a row to modify, so add this data as a new row
                                        return Some(scan_row);
                                    });
                                }
                                self.table.recreate_rows();
                            }
                        }
                    }
                    msg => {
                        info!("TODO: handle msg = {msg:?}");
                    }
                }
            }

            ui.label(format!("State: {:?}", self.bt_state));
            ui.label(format!("Found {} devices", self.scan_map.len()));

            // scan start/stop
            ui.horizontal(|ui| {
                let start_enabled = matches!(self.bt_state, AmReadyIdle(_));
                let stop_enabled = matches!(self.bt_state, AmScanning);
                ui.add_enabled_ui(start_enabled, |ui| {
                    if ui.button("Start Scan").clicked() {
                        // TODO: provide actual scan options into DoScanStart message from UI
                        let send_start_scan_res = self.cmd_tx.send(DoScanStart("".into()));
                        info!("send_start_scan_res = {send_start_scan_res:?}");
                    }
                });
                ui.add_enabled_ui(stop_enabled, |ui| {
                    if ui.button("Stop Scan").clicked() {
                        let send_stop_scan_res = self.cmd_tx.send(DoScanStop);
                        info!("send_stop_scan_res = {send_stop_scan_res:?}");
                    }
                });
                if ui.button("Clear Scan List").clicked() {
                    self.scan_map.clear();
                    self.scan_vec.clear();
                    self.table.clear_all_rows();
                    self.table.recreate_rows();
                }
            });

            let work_row_id = self.table.config.connect_row_id;
            if let Some(connect_row_id) = work_row_id {
                // latch variable out
                self.table.config.connect_row_id = None;

                let row_map = self.table.get_all_rows();
                match row_map.get(&connect_row_id) {
                    Some(connect_row) => {
                        let row_data = connect_row.row_data.clone();
                        info!(
                            "Should connect to name={}, rssi={}, bt_id={:?}", //
                            row_data.name, row_data.rssi, row_data.bt_id
                        );

                        match row_data.bt_id {
                            Some(bt_id) => {
                                let _ = self.cmd_tx.send(DoConnect(bt_id));
                                self.bt_state = AmConnecting;
                            }
                            None => {
                                warn!("Couldn't get bt_id from select/connect row_data");
                            }
                        }
                    }
                    None => {
                        warn!("Couldn't resolve row id {connect_row_id}");
                    }
                }
                return;
            }

            self.table.show_ui(ui, |table| {
                let mut table = table
                    .drag_to_scroll(false)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .drag_to_scroll(false)
                    .auto_shrink([false; 2])
                    .min_scrolled_height(0.0);

                for _col in ScanColumns::iter() {
                    // table = table.column(Column::initial(150.0))
                    table = table.column(Column::auto())
                }
                table
            });
        });
        //
    }
}

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

    let options = eframe::NativeOptions::default();
    eframe::run_native("", options, Box::new(|cc| Ok(Box::new(NusGui::new(cc)))))
}

fn scan_obj_to_scan_row(scan_obj: &AdvertisingDevice) -> ScanRow {
    ScanRow {
        bt_id: Some(scan_obj.device.id()),
        name: scan_obj.device.name().unwrap_or("n/a".into()),
        rssi: scan_obj.rssi.unwrap_or(-200_i16),
    }
}
