use tracing::Level;

mod components;

use components::app::App;

fn main() {
    console_error_panic_hook::set_once();
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}
