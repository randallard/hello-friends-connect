use leptos::*;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, Event, MouseEvent};

use crate::connect_component::{Connection, ConnectionStatus};
use crate::connection_modal::ConnectionModal;

#[component]
pub fn ConnectionItem(
    #[prop(into)] connection: Connection,
    #[prop(into)] name: String,
    #[prop(optional)] on_delete: Option<Callback<String>>,
) -> impl IntoView {
    let connection_signal = create_rw_signal(connection);
    let connection_id = create_memo(move |_| connection_signal.get().id.clone());
    let connection_name = create_rw_signal(name);
    let show_view_modal = create_rw_signal(false);
    let show_expired_modal = create_rw_signal(false);
    
    // Create a proper derived signal for status that will update reactively
    let status = create_memo(move |_| connection_signal.get().status.clone());

    let console_log = move |msg: &str| {
        console::log_1(&wasm_bindgen::JsValue::from_str(msg));
    };

    let _ = create_effect(move |_| {
        console_log(&format!("Initial status set to: {:?}", status.get()));
    });

    // // Debug log the initial connection state
    // console_log(&format!("ConnectionItem initialized with connection ID: {} and status: {:?}", 
    //                      connection.id, connection.status));

    Effect::new(move |_| {
        let conn = connection_signal.get();
        let new_status = status.get();
        console_log(&format!("Effect: Connection status updated for {}: {:?}", conn.id, new_status));
    });                         

    // Create a signal to track if this component is still valid
    // This helps prevent errors when trying to access deleted connections
    let is_valid = create_rw_signal(true);
    
    // Function to handle status button click
    let handle_status_click = move |_| {
        match status.get() {
            ConnectionStatus::Expired => {
                show_expired_modal.set(true);
            },
            ConnectionStatus::Pending | ConnectionStatus::Active => {
                show_view_modal.set(true);
            }
        }
    };
    
    // Function to handle refresh action
    let handle_refresh = move |_: MouseEvent| {
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
            &format!("Refreshing connection: {}", connection_id.get())
        ));
        show_expired_modal.set(false);
    };
    
    // Function to handle delete action
    let handle_delete = move |_: MouseEvent| {
        if !is_valid.get() {
            return; // Skip if already deleted
        }
        
        let conn_id = connection_id.get();
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
            &format!("Deleting connection: {}", conn_id)
        ));
        
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
        
        // Mark this component as invalid before calling delete callback
        is_valid.set(false);
        
        // Close any open modals
        show_expired_modal.set(false);
        show_view_modal.set(false);
        
        // Call the delete callback if provided
        if let Some(callback) = on_delete {
            callback.run(conn_id);
        }
    };
    
    view! {
        <div class="flex justify-between items-center p-3 border-b border-gray-700 last:border-b-0">
            {move || {
                // Skip rendering if this component is no longer valid
                if !is_valid.get() {
                    return view! { <></> }.into_any();
                }
                
                view! {
                    <>
                        <div class="font-medium">{move || connection_name.get()}</div>
                        <div>
                            - / -
                            <button 
                                data-test-id="connection-status"
                                class={move || match status.get() {
                                    ConnectionStatus::Pending => "px-3 py-1 bg-yellow-600 hover:bg-yellow-700 rounded text-sm text-gray-100",
                                    ConnectionStatus::Active => "px-3 py-1 bg-green-600 hover:bg-green-700 rounded text-sm text-gray-100", 
                                    ConnectionStatus::Expired => "px-3 py-1 bg-red-600 hover:bg-red-700 rounded text-sm text-gray-100",
                                }}
                                on:click=handle_status_click
                            >
                                {move || {
                                    let conn_id = connection_id.get();
                                    let current_status = status.get();
                                    
                                    match current_status {
                                        ConnectionStatus::Pending => {
                                            console_log(&format!("Rendering Pending status for connection: {}", conn_id));
                                            "Pending"
                                        },
                                        ConnectionStatus::Active => {
                                            console_log(&format!("Rendering Active status for connection: {}", conn_id));
                                            "Active"
                                        },
                                        ConnectionStatus::Expired => {
                                            console_log(&format!("Rendering Expired status for connection: {}", conn_id));
                                            "Expired"
                                        },
                                    }
                                }}
                            </button>
                        </div>
                    </>
                }.into_any()
            }}
            
            // Fix modal for expired connections - properly wrap reactive signals
            {move || {
                if show_expired_modal.get() && is_valid.get() {
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
                                        data-test-id="cancel-connection-button"
                                        class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-gray-200"
                                        on:click=move |_| show_expired_modal.set(false)
                                    >
                                        "Cancel"
                                    </button>
                                    <button
                                        data-test-id="delete-connection-button"
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
    
            // View modal for connections - fix to ensure link_id is displayed
            {move || {
                if show_view_modal.get() && is_valid.get() {
                    // Get the current connection from the signal
                    let current_connection = connection_signal.get();
                    let name_value = connection_name.get();
                    
                    view! {
                        <ConnectionModal
                            connection_name=create_signal(name_value).0
                            show_name_error=create_signal(false).0
                            is_view_mode=true
                            connection_link_id=current_connection.link_id.clone()
                            on_name_change=Callback::new(move |_| {
                                // No-op for view mode
                            })
                            on_cancel=Callback::new(move |_| {
                                show_view_modal.set(false);
                            })
                            on_delete=Callback::new(move |_| {
                                handle_delete(web_sys::MouseEvent::new("click").unwrap());
                            })
                            on_submit=Callback::new(move |_| {
                                handle_refresh(web_sys::MouseEvent::new("click").unwrap());
                                show_view_modal.set(false);
                            })
                        />
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::*;
    use js_sys::{Function, Promise};

    wasm_bindgen_test_configure!(run_in_browser);

    // Setup mock for WebSocket operations
    fn setup_websocket_mock() {
        let window = web_sys::window().unwrap();
        
        // Create mock WebSocket constructor
        let mock_websocket_constructor = Function::new_with_args(
            "url",
            r#"
            console.log('Mock WebSocket created with URL:', url);
            
            // Create a mock WebSocket object
            const mockWs = {
                url: url,
                readyState: 1, // WebSocket.OPEN
                send: function(data) {
                    console.log('Mock WebSocket sent data:', data);
                },
                close: function() {
                    console.log('Mock WebSocket closed');
                    this.readyState = 3; // WebSocket.CLOSED
                    if (this.onclose) this.onclose({});
                }
            };
            
            // Simulate connection open on next tick
            setTimeout(() => {
                if (mockWs.onopen) mockWs.onopen({});
            }, 0);
            
            return mockWs;
            "#
        );
        
        // Store original WebSocket constructor
        let _ = js_sys::Reflect::set(
            &window, 
            &JsValue::from_str("originalWebSocket"), 
            &window.get("WebSocket").unwrap()
        );
        
        // Replace with mock
        let _ = js_sys::Reflect::set(
            &window, 
            &JsValue::from_str("WebSocket"), 
            &mock_websocket_constructor
        );
    }
    
    // Restore original WebSocket after test
    fn restore_websocket() {
        if let Some(window) = web_sys::window() {
            if let Ok(orig_ws) = js_sys::Reflect::get(&window, &JsValue::from_str("originalWebSocket")) {
                if !orig_ws.is_undefined() {
                    let _ = js_sys::Reflect::set(&window, &JsValue::from_str("WebSocket"), &orig_ws);
                }
            }
        }
    }

    #[wasm_bindgen_test]
    async fn test_connection_status_button_changes() {
        // Setup WebSocket mock before test
        setup_websocket_mock();
        
        // Create cleanup guard
        struct CleanupGuard;
        impl Drop for CleanupGuard {
            fn drop(&mut self) {
                restore_websocket();
            }
        }
        let _guard = CleanupGuard;
        
        // Test with a Pending connection first
        let pending_connection = Connection {
            id: "test-conn-123".to_string(),
            link_id: "test-link-456".to_string(),
            players: vec!["player1".to_string()],
            created_at: js_sys::Date::now() as i64 / 1000,
            status: ConnectionStatus::Pending,
            expires_at: (js_sys::Date::now() as i64 / 1000) + 86400,
        };
        
        // Mount the component
        mount_to_body(move || view! {
            <ConnectionItem
                connection=pending_connection.clone()
                name="Test Connection"
                on_delete=Callback::new(|_| {})
            />
        });
        
        // Verify Pending state
        let status = document()
            .query_selector("[data-test-id='connection-status']")
            .unwrap()
            .expect("Should find connection status element");
        
        assert_eq!(status.text_content().unwrap().trim(), "Pending");
    }

    
    #[wasm_bindgen_test]
    async fn test_active_connection_status() {
        // Setup WebSocket mock before test
        setup_websocket_mock();
        
        // Create cleanup guard
        struct CleanupGuard;
        impl Drop for CleanupGuard {
            fn drop(&mut self) {
                restore_websocket();
            }
        }
        let _guard = CleanupGuard;
    
        // Connection name for testing
        let connection_name = "Test Active Connection";
    
        // Mount with the Active connection
        mount_to_body(move || {         
            // Create an Active connection
            let active_connection = Connection {
                id: "test-conn-123".to_string(),
                link_id: "test-link-456".to_string(),
                players: vec!["player1".to_string(), "player2".to_string()],
                created_at: js_sys::Date::now() as i64 / 1000,
                status: ConnectionStatus::Active,
                expires_at: (js_sys::Date::now() as i64 / 1000) + 86400,
            };
            view! {<ConnectionItem
                connection=active_connection
                name=connection_name.to_string()
                on_delete=Callback::new(|_| {})
            />}
        });
        
        // Wait longer for the DOM to update and reactive signals to process
        let _ = gloo_timers::future::TimeoutFuture::new(500).await;
        
        // Get window and document
        let window = web_sys::window().expect("should have a window");
        let document = window.document().expect("should have a document");
        
        // Get all status elements
        let all_status_elements = document
            .query_selector_all("[data-test-id='connection-status']")
            .expect("should be able to query status elements");
            
        let total_count = all_status_elements.length();
        web_sys::console::log_1(&JsValue::from_str(
            &format!("Found {} total status elements", total_count)
        ));
        
        // Find both connection name and status text for each element
        for i in 0..total_count {
            if let Some(element) = all_status_elements.item(i) {
                if let Ok(el) = element.dyn_into::<web_sys::Element>() {
                    // Find the parent container that has the connection name
                    if let Some(parent_container) = el.closest(".flex.justify-between").ok().flatten() {
                        if let Some(name_element) = parent_container
                            .query_selector(".font-medium")
                            .ok()
                            .flatten() 
                        {
                            let conn_name = name_element.text_content().unwrap_or_default();
                            let status_text = el.text_content().unwrap_or_default();
                            
                            web_sys::console::log_1(&JsValue::from_str(
                                &format!("Element #{}: Connection '{}' has status '{}'", 
                                        i + 1, conn_name.trim(), status_text.trim())
                            ));
                            
                            // Add additional check - if the connection has our test name,
                            // verify it has the correct status
                            if conn_name.trim() == connection_name {
                                assert_eq!(
                                    status_text.trim(), 
                                    "Active", 
                                    "Test connection should have Active status"
                                );
                                
                                // Also check styling
                                let class_attr = el.get_attribute("class").unwrap_or_default();
                                assert!(
                                    class_attr.contains("bg-green-600"), 
                                    "Active element should have green background class"
                                );
                            }
                        }
                    }
                }
            }
        }
        
        // Assert that we found at least one element (already checked status above)
        assert!(total_count > 0, "Should find at least one status element");
    }

    #[wasm_bindgen_test]
    async fn test_connection_view_modal() {
        // Setup WebSocket mock
        setup_websocket_mock();
        
        // Create cleanup guard
        struct CleanupGuard;
        impl Drop for CleanupGuard {
            fn drop(&mut self) {
                restore_websocket();
            }
        }
        let _guard = CleanupGuard;
        
        // Create a test connection with known values
        let test_connection = Connection {
            id: "test-conn-456".to_string(),
            link_id: "test-link-789".to_string(),
            players: vec!["player1".to_string()],
            created_at: js_sys::Date::now() as i64 / 1000,
            status: ConnectionStatus::Pending,
            expires_at: (js_sys::Date::now() as i64 / 1000) + 86400,
        };
        
        // Mount the component
        mount_to_body(move || view! {
            <ConnectionItem
                connection=test_connection.clone()
                name="Test Connection"
                on_delete=Callback::new(|_| {})
            />
        });
        
        // Wait for component to render
        let _ = gloo_timers::future::TimeoutFuture::new(100).await;
        
        // Click the status button to open the view modal
        let status_button = document()
            .query_selector("[data-test-id='connection-status']")
            .unwrap()
            .expect("Should find status button");
            
        status_button.dispatch_event(&web_sys::Event::new("click").unwrap()).unwrap();
        
        // Wait for modal to appear
        let _ = gloo_timers::future::TimeoutFuture::new(100).await;
        
        // Verify the modal shows the connection link ID
        let link_input = document()
            .query_selector(".bg-gray-800 input[readonly]")
            .unwrap()
            .expect("Should find read-only link input");
            
        let input_value = link_input
            .dyn_ref::<web_sys::HtmlInputElement>()
            .expect("Should be an input element")
            .value();

        web_sys::console::log_1(&JsValue::from_str(
            &format!("Link value: {}", input_value)
        ));
            
        // Verify link ID is in the input value
        assert!(input_value.contains("test-link"), 
                "Modal should display the connection link ID");
    }
}