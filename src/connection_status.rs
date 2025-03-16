// connection_status.rs
use leptos::*;
use leptos::prelude::*;

#[component]
pub fn ConnectionStatus(
    #[prop(into)] connected: Signal<bool>,
) -> impl IntoView {
    // Add a derived signal to show connecting state
    let (connecting, set_connecting) = create_signal(false);
    
    // Create an effect to handle initial connection attempts
    create_effect(move |_| {
        if !connected.get() {
            let timer_id = set_timeout(move || {
                // Clear connecting state after 5 seconds if still not connected
                set_connecting.set(false);
            }, 
            std::time::Duration::from_millis(5000));
        }
    });
    
    view! {
        <div 
            class="inline-flex items-center"
            title=move || {
                if connected.get() {
                    "Connected to server"
                } else if connecting.get() {
                    "Connecting to server..."
                } else {
                    "Disconnected from server"
                }
            }
        >
            <div 
                class=move || {
                    if connected.get() {
                        "w-3 h-3 rounded-full bg-green-500 animate-pulse"
                    } else if connecting.get() {
                        "w-3 h-3 rounded-full bg-yellow-500 animate-pulse"
                    } else {
                        "w-3 h-3 rounded-full bg-red-500"
                    }
                }
            >
            </div>
        </div>
    }
}