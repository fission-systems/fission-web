use tracing::Level;

mod components;

use components::app::App;

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}
