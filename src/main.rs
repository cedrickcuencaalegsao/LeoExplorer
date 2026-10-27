use std::sync::{Arc, mpsc};
use shared::{CacheEvent, Reporter};

fn main() -> iced::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,cache-tree=trace")).init();

    let (_tx, _rx) = mpsc::channel::<CacheEvent>();

    let report: Reporter = Arc::new(move |event| {
        let _ = _tx.send(event);
    });

    cache::init(report);

    // hand `rx` to the GUI, e.g. via a static/OnceLock or a custom run function
    leoexplorer_gui::run()
}