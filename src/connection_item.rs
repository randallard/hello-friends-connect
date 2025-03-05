use leptos::*;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::connect_component::{Connection, ConnectionStatus};

#[component]
pub fn ConnectionItem(
    #[prop(into)] connection: Connection,
    #[prop(into)] name: String,
    #[prop(optional)] on_delete: Option<Callback<String>>,
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
        
        // Get the connection ID to delete
        let conn_id = connection_id.get();
        
        // Remove from localStorage
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                // Remove the connection name
                let name_key = format!("conn-name-{}", conn_id);
                let _ = storage.remove_item(&name_key);
                
                // Remove from saved connections
                if let Ok(Some(saved_json)) = storage.get_item("saved-connections") {
                    if let Ok(mut saved_connections) = serde_json::from_str::<Vec<serde_json::Value>>(&saved_json) {
                        // Filter out the connection to delete
                        saved_connections.retain(|conn| {
                            if let Some(id) = conn.get("id").and_then(|v| v.as_str()) {
                                id != conn_id
                            } else {
                                true // Keep entries without an id
                            }
                        });
                        
                        // Save the updated list
                        if let Ok(updated_json) = serde_json::to_string(&saved_connections) {
                            let _ = storage.set_item("saved-connections", &updated_json);
                        }
                    }
                }
                
                // Also remove from connection names map if it exists
                if let Ok(Some(names_json)) = storage.get_item("connection-names") {
                    if let Ok(mut names_map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&names_json) {
                        names_map.remove(&conn_id);
                        if let Ok(updated_json) = serde_json::to_string(&names_map) {
                            let _ = storage.set_item("connection-names", &updated_json);
                        }
                    }
                }
            }
        }
        
        // Call the delete callback if provided
        if let Some(callback) = on_delete {
            callback.run(conn_id);
        }
        
        // Close the modal
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