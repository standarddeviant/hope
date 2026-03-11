use egui_selectable_table::{
    AutoScroll, ColumnOperations, ColumnOrdering, SelectableRow, SelectableTable, SortOrder,
};

use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter}; // 0.25

use egui::{Button, Ui};

#[derive(Default, Clone, Copy)]
pub struct Config {
    // counting_ongoing: bool,
}

#[derive(Clone, Default)]
pub struct ScanRow {
    pub id: String,
    // addr: String,
    pub name: String,
    pub rssi: i16,
}

#[derive(Eq, PartialEq, Debug, Ord, PartialOrd, Clone, Copy, Hash, Default, EnumIter, Display)]
pub enum ScanColumns {
    #[default]
    #[strum(to_string = "Id")]
    Id,
    // #[strum(to_string = "Address")]
    // Address,
    #[strum(to_string = "Name")]
    Name,
    #[strum(to_string = "RSSI")]
    RSSI,
}

impl ColumnOperations<ScanRow, ScanColumns, Config> for ScanColumns {
    fn column_text(&self, row: &ScanRow) -> String {
        match self {
            ScanColumns::Id => row.id.to_string(),
            // ScanColumns::Address => row.addr.to_string(),
            ScanColumns::Name => row.name.to_string(),
            ScanColumns::RSSI => row.rssi.to_string(),
        }
    }
    fn create_header(
        &self,
        ui: &mut Ui,
        sort_order: Option<SortOrder>,
        _table: &mut SelectableTable<ScanRow, ScanColumns, Config>,
    ) -> Option<egui::Response> {
        let mut text = self.to_string();

        if let Some(sort) = sort_order {
            match sort {
                SortOrder::Ascending => text += "🔽",
                SortOrder::Descending => text += "🔼",
            }
        }
        let selected = sort_order.is_some();
        let resp = ui.add_sized(ui.available_size(), Button::selectable(selected, text));
        Some(resp)
    }
    fn create_table_row(
        &self,
        ui: &mut Ui,
        row: &SelectableRow<ScanRow, ScanColumns>,
        cell_selected: bool,
        table: &mut SelectableTable<ScanRow, ScanColumns, Config>,
    ) -> egui::Response {
        let row_id = row.id;
        let row_data = &row.row_data;
        let config = table.config;

        let text = match self {
            ScanColumns::Id => row_data.id.to_string(),
            // ScanColumns::Address => row_data.addr.to_string(),
            ScanColumns::Name => row_data.name.to_string(),
            ScanColumns::RSSI => row_data.rssi.to_string(),
        };

        // Persist the creation count, while row creation is ongoing, this will get auto
        // reloaded. After there is no more row creation, auto reload is turned off and won't
        // reload until next manual intervention. While no more rows are being created, we are
        // modifying the rows directly that are being shown in the UI which is much less
        // expensive and gets shown to the UI immediately.
        // Continue to update the persistent row data to ensure once reload happens, the
        // previous count data is not lost
        table.add_modify_row(|table| {
            let target_row = table.get_mut(&row_id).unwrap();
            // target_row.row_data.create_count += 1;
            None
        });
        // if !config.counting_ongoing {
        //     table.modify_shown_row(|t, index| {
        //         let target_index = index.get(&row_id).unwrap();
        //         let target_row = t.get_mut(*target_index).unwrap();
        //         target_row.row_data.create_count += 1;
        //     });
        // }

        // The same approach works for both cell based selection and for entire row selection on
        // drag.
        let resp = ui.add_sized(ui.available_size(), Button::selectable(cell_selected, text));

        resp.context_menu(|ui| {
            if ui.button("Select All Rows").clicked() {
                table.select_all();
                ui.close();
            }
            if ui.button("Unselect All Rows").clicked() {
                table.unselect_all();
                ui.close();
            }
            if ui.button("Copy Selected Cells").clicked() {
                table.copy_selected_cells(ui);
                ui.close();
            }
            if ui.button("Mark row as selected").clicked() {
                ui.close();
                table.mark_row_as_selected(row_id, None);
            }
        });
        resp
    }
}

impl ColumnOrdering<ScanRow> for ScanColumns {
    fn order_by(&self, row_1: &ScanRow, row_2: &ScanRow) -> std::cmp::Ordering {
        match self {
            ScanColumns::Id => row_1.id.cmp(&row_2.id),
            // ScanColumns::Address => row_1.addr.cmp(&row_2.addr),
            ScanColumns::Name => row_1.name.cmp(&row_2.name),
            ScanColumns::RSSI => row_1.rssi.cmp(&row_2.rssi),
        }
    }
}
