//! Sidebar — function list with search filter.

use crate::engine::decompile_function;
use crate::state::{use_app_state, LogEntry};
use dioxus::prelude::*;

#[component]
pub fn Sidebar() -> Element {
    let mut state = use_app_state();
    let search = state.read().sidebar_search.clone();
    let selected = state.read().current_fn_addr;

    let functions: Vec<_> = {
        let s = state.read();
        let q = s.sidebar_search.to_lowercase();
        s.functions.iter()
            .filter(|f| {
                q.is_empty()
                    || f.name.to_lowercase().contains(&q)
                    || format!("{:x}", f.address).contains(&q)
            })
            .cloned()
            .collect()
    };

    rsx! {
        div { class: "sidebar",
            div { class: "sidebar-header",
                span { class: "sidebar-title", "Functions" }
                span { class: "fn-badge", "{functions.len()}" }
            }
            div { class: "sidebar-search",
                input {
                    r#type: "text",
                    class: "search-input",
                    placeholder: "Search functions…",
                    value: "{search}",
                    oninput: move |e| state.write().sidebar_search = e.value(),
                }
            }
            ul { class: "function-list",
                for f in functions.iter() {
                    {
                        let is_sel = selected == Some(f.address);
                        let name = if f.name.is_empty() {
                            format!("sub_{:x}", f.address)
                        } else {
                            f.name.clone()
                        };
                        let addr = f.address;
                        let dot_cls = if f.is_export { "fn-dot is-export" }
                                      else if f.is_import || f.is_thunk { "fn-dot is-import" }
                                      else { "fn-dot is-code" };
                        let item_cls = if is_sel { "function-item is-selected" } else { "function-item" };
                        let name2 = name.clone();
                        rsx! {
                            li {
                                key: "{addr}",
                                class: "{item_cls}",
                                onclick: move |_| {
                                    {
                                        let mut s = state.write();
                                        s.current_fn_addr = Some(addr);
                                        s.navigate_to(addr);
                                        s.decompiled_code = None;
                                        s.decompiled_nir  = None;
                                        s.push_log(LogEntry::info(
                                            format!("Decompiling {name2}  @  0x{addr:x}…")
                                        ));
                                    }
                                    decompile_function(addr, name.clone(), state);
                                },
                                div { class: "{dot_cls}" }
                                div { class: "fn-info",
                                    div { class: "fn-name", "{name}" }
                                    div { class: "fn-addr", "0x{addr:016x}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
