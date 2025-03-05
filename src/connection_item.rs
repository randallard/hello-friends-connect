use leptos::*;
use leptos::prelude::*;

use crate::connect_component::{Connection, ConnectionStatus};

#[component]
pub fn ConnectionItem(
    #[prop(into)] connection: Connection,
    #[prop(into)] name: String,
) -> impl IntoView {
    // Create local clone of connection values to avoid ownership issues
    let status = create_rw_signal(connection.status);
    let connection_id = create_rw_signal(connection.id.clone());
    let connection_name = create_rw_signal(name);
    
    // Signal to track if the confirmation modal is visible
    let show_expired_modal = create_rw_signal(false);
    
    // Function to handle refresh action
    let handle_refresh = move |_| {
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
            &format!("Refreshing connection: {}", connection_id.get())
        ));
        show_expired_modal.set(false);
    };
    
    // Function to handle delete action
    let handle_delete = move |_| {
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
            &format!("Deleting connection: {}", connection_id.get())
        ));
        show_expired_modal.set(false);
    };
    
    // Function to handle status button click
    let handle_status_click = move |_| {
        if status.get() == ConnectionStatus::Expired {
            show_expired_modal.set(true);
        }
    };
    
    view! {
        <div class="flex justify-between items-center p-3 border-b border-gray-700 last:border-b-0">
            <div class="font-medium">{connection_name}</div>
            <div>
                - / -
                <button 
                    class={move || match status.get() {
                        ConnectionStatus::Pending => "px-3 py-1 bg-yellow-600 hover:bg-yellow-700 rounded text-sm text-gray-100",
                        ConnectionStatus::Active => "px-3 py-1 bg-green-600 hover:bg-green-700 rounded text-sm text-gray-100", 
                        ConnectionStatus::Expired => "px-3 py-1 bg-red-600 hover:bg-red-700 rounded text-sm text-gray-100",
                    }}
                    on:click=handle_status_click
                >
                    {move || match status.get() {
                        ConnectionStatus::Pending => "Pending",
                        ConnectionStatus::Active => "Active",
                        ConnectionStatus::Expired => "Expired",
                    }}
                </button>
            </div>
            
            // Modal for expired connections
            {move || {
                if show_expired_modal.get() {
                    view! {
                        <div class="fixed inset-0 bg-black bg-opacity-70 flex items-center justify-center z-50">
                            <div class="bg-gray-800 p-6 rounded-lg shadow-xl max-w-md w-full mx-4 text-gray-100 border border-gray-700">
                                <h3 class="text-xl font-bold mb-4 text-gray-100">
                                    "Connection Expired"
                                </h3>
                                <p class="mb-6 text-gray-300">
                                    "This connection has expired. What would you like to do?"
                                </p>
                                
                                <div class="flex justify-end gap-3">
                                    <button
                                        class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-gray-200"
                                        on:click=move |_| show_expired_modal.set(false)
                                    >
                                        "Cancel"
                                    </button>
                                    <button
                                        class="px-4 py-2 border border-red-500 text-red-500 hover:bg-red-900 rounded"
                                        on:click=handle_delete.clone()
                                    >
                                        "Delete"
                                    </button>
                                    <button
                                        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-gray-100"
                                        on:click=handle_refresh.clone()
                                    >
                                        "Refresh"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }
            }}
        </div>
    }
}