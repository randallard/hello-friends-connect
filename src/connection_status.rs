// connection_status.rs
use leptos::*;
use leptos::prelude::*;

#[component]
pub fn ConnectionStatus(
    #[prop(into)] connected: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="inline-flex items-center ml-2">
            <div 
                class=move || {
                    if connected.get() {
                        "w-3 h-3 rounded-full bg-green-500 animate-pulse"
                    } else {
                        "w-3 h-3 rounded-full bg-red-500"
                    }
                }
                title=move || {
                    if connected.get() {
                        "Connected to server"
                    } else {
                        "Disconnected from server"
                    }
                }
            >
            </div>
            <span class="ml-1 text-xs text-gray-400">
                {move || if connected.get() { "Connected" } else { "Disconnected" }}
            </span>
        </div>
    }
}