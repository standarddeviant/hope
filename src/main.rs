use eframe::egui;
use egui::CentralPanel;
use egui_inbox::UiInbox;

use tokio::runtime::Runtime;

mod btnus;
use btnus::spawn_btnus_thread;

pub fn main() -> eframe::Result<()> {
    let inbox = UiInbox::new();
    let mut state = None;
    let tx = inbox.sender();

    eframe::run_simple_native(
        "DnD Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                // `read` will return an iterator over all pending messages
                if let Some(last) = inbox.read(ui).last() {
                    state = last;
                }
                // There also is a `replace` method that you can use as a shorthand for the above:
                // inbox.replace(ui, &mut state);

                ui.label(format!("State: {:?}", state));
                if ui.button("Async Task").clicked() {
                    state = Some("Waiting for async task to complete".to_string());
                    let tx = inbox.sender();
                    spawn_btnus_thread(tx);
                }
            });
        },
    )
}
