use leptos::*;
use leptos::prelude::*;  
use serde::{Serialize, Deserialize};
use std::ops::Not;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

use crate::connection_modal::ConnectionModal; 
use crate::connection_utils;
use crate::connection_item::ConnectionItem;

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionModalMode {
    Add,
    View,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    Pending,
    Active,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub link_id: String, 
    pub players: Vec<String>,
    pub created_at: i64,
    pub status: ConnectionStatus,
    pub expires_at: i64,
}

#[component]
pub fn FriendsConnect() -> impl IntoView {
    let (show_connection, set_show_connection) = signal(false);
    let (connection_name, set_connection_name) = signal(String::new());
    let (show_name_error, set_show_name_error) = signal(false);
    let (api_error, set_api_error) = signal(String::new());
    let (current_connection, set_current_connection) = signal(None::<Connection>);

    // Signal for active connections
    let (connections, set_connections) = signal(Vec::<Connection>::new());

    // Helper for logging
    let console_log = move |msg: &str| {
        console::log_1(&wasm_bindgen::JsValue::from_str(msg));
    };

    // Effect to ensure a player ID exists
    Effect::new(move |_| {
        if get_stored_player_id().is_none() {
            // No player ID exists, create one
            let window = web_sys::window().expect("no global window exists");
            let storage = window.local_storage()
                .expect("failed to get localStorage")
                .expect("localStorage not available");
                
            let new_id = uuid::Uuid::new_v4().to_string();
            storage.set_item("player-id", &new_id)
                .expect("failed to store player-id");
        }
    });

    // Effect to check for link ID in URL
    Effect::new(move |_| {
        if let Some(link_id) = connection_utils::get_link_id_from_url() {
            console_log(&format!("Found link ID in URL: {}", link_id));
            
            // Auto-open the connection modal
            set_show_connection.set(true);
        }
    });

    Effect::new(move |_| {
        // Use the existing load_saved_connections function
        let saved_connections = connection_utils::load_saved_connections();
        
        if !saved_connections.is_empty() {
            console_log("Loading saved connections from local storage");
            
            // Convert saved connections to Connection objects and add to connections signal
            for saved_conn in saved_connections {
                if let (Some(id), Some(link_id), Some(created_at)) = (
                    saved_conn.get("id").and_then(|v| v.as_str()),
                    saved_conn.get("link_id").and_then(|v| v.as_str()),
                    saved_conn.get("created_at").and_then(|v| v.as_i64())
                ) {
                    // Get expires_at from saved connection or calculate it if not present
                    let expires_at = saved_conn.get("expires_at")
                        .and_then(|v| v.as_i64())
                        .unwrap_or_else(|| created_at + 86400000); // 24 hours from creation
                    
                    // Get expires_at from saved connection or calculate it if not present
                    let expires_at = saved_conn.get("expires_at")
                        .and_then(|v| v.as_i64())
                        .unwrap_or_else(|| created_at + 86400); // 24 hours from creation in seconds

                    // Convert expires_at to milliseconds for comparison with js_sys::Date::now()
                    let expires_at_ms = expires_at * 1000;

                    // Set status based on expiration time
                    let status = if expires_at_ms > js_sys::Date::now() as i64 {
                        ConnectionStatus::Pending
                    } else {
                        ConnectionStatus::Expired
                    };
                    
                    let connection = Connection {
                        id: id.to_string(),
                        link_id: link_id.to_string(),
                        players: Vec::new(), // We don't store this in localStorage
                        created_at,
                        status,
                        expires_at,
                    };
                    
                    // Add to connections list if not already present
                    set_connections.update(|conns| {
                        if !conns.iter().any(|c| c.id == connection.id) {
                            conns.push(connection);
                        }
                    });
                }
            }
        }
    });

    let create_connection = move || {
        let name = connection_name.get();
        if name.trim().is_empty() {
            set_show_name_error.set(true);
            return;
        }
    
        let player_id = get_stored_player_id().unwrap_or_else(|| {
            let new_id = uuid::Uuid::new_v4().to_string();
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("player-id", &new_id);
                }
            }
            new_id
        });
    
        let name_clone = name.clone();
        console_log(&format!("Creating connection with name: {}", name));
        
        // Reset error state
        set_api_error.set(String::new());
        
        spawn_local(async move {
            match connection_utils::create_connection(&player_id).await {
                Ok(connection) => {
                    console_log(&format!("Connection created with ID: {} and link_id: {}", 
                        connection.id, connection.link_id));
                    
                    // Save friendly name for this connection
                    if let Some(window) = web_sys::window() {
                        if let Ok(Some(storage)) = window.local_storage() {
                            let _ = storage.set_item(&format!("conn-name-{}", connection.id), &name_clone);
                        }
                    }
                    
                    // Save the connection for later
                    let _ = connection_utils::save_connection_to_local_storage(&connection, &name_clone);
                    
                    // Update current connection
                    set_current_connection.set(Some(connection.clone()));
                    
                    // Add to connections list
                    set_connections.update(|conns| {
                        conns.push(connection);
                    });
                    
                    // Close the modal
                    set_show_connection.set(false);
                    set_connection_name.set(String::new());
                },
                Err(e) => {
                    let error_msg = format!("Error creating connection: {:?}", e);
                    console_log(&error_msg);
                    set_api_error.set(error_msg);
                }
            }
        });
    };

    // Join a connection with the API
    let join_connection = move |link_id: String| {
        let name = connection_name.get();
        if name.trim().is_empty() {
            set_show_name_error.set(true);
            return;
        }

        let player_id = get_stored_player_id().unwrap_or_else(|| {
            let new_id = uuid::Uuid::new_v4().to_string();
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("player-id", &new_id);
                }
            }
            new_id
        });

        let name_clone = name.clone();
        console_log(&format!("Joining connection with link ID: {}", link_id));
        
        // Reset error state
        set_api_error.set(String::new());
        
        spawn_local(async move {
            match connection_utils::join_connection(&link_id, &player_id).await {
                Ok(connection) => {
                    console_log(&format!("Connection joined: {}", connection.id));
                    
                    // Save friendly name for this connection
                    if let Some(window) = web_sys::window() {
                        if let Ok(Some(storage)) = window.local_storage() {
                            let _ = storage.set_item(&format!("conn-name-{}", connection.id), &name_clone);
                        }
                    }
                    
                    // Save the connection for later
                    let _ = connection_utils::save_connection_to_local_storage(&connection, &name_clone);
                    
                    // Update current connection
                    set_current_connection.set(Some(connection));
                    
                    // Close the modal
                    set_show_connection.set(false);
                    set_connection_name.set(String::new());
                },
                Err(e) => {
                    let error_msg = format!("Error joining connection: {:?}", e);
                    console_log(&error_msg);
                    set_api_error.set(error_msg);
                }
            }
        });
    };

    view! {
        <div id="friends-connect-container" class="max-w-md mx-auto p-4 bg-gray-900 text-gray-100">
            <h2 class="text-xl font-bold mb-4 text-gray-100">"Connect with Friends"</h2>
            
            {move || {
                if let Some(err) = api_error.get().as_str().is_empty().not().then(|| api_error.get()) {
                    view! {
                        <div class="bg-red-900 text-red-100 p-4 rounded mb-4">
                            {err}
                        </div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }
            }}
            
            <button
                class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 rounded text-gray-100 mb-4"
                on:click=move |_| {
                    set_show_name_error.set(false); 
                    set_show_connection.set(true);
                }
            >
                "New Connection"
            </button>

            // Display the list of connections
            <div class="mt-4">
                {move || {
                    let connections_list = connections.get();
                    if connections_list.is_empty() {
                        view! {
                            <div class="text-gray-400 text-sm mt-2">
                                "No connections yet. Click 'New Connection' to create one."
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="border border-gray-700 rounded overflow-hidden">
                                // Update the ConnectionItem rendering in FriendsConnect
                                <For
                                    each=move || connections.get()
                                    key=|conn| conn.id.clone()
                                    let:connection
                                >
                                    {move || {
                                        let conn_id = connection.id.clone();
                                        let name = get_connection_name(&conn_id).unwrap_or_else(|| "Unnamed Connection".to_string());
                                        view! {
                                            <ConnectionItem 
                                                connection=connection.clone() 
                                                name=name 
                                                on_delete=Callback::new(move |deleted_id: String| {
                                                    // Remove the deleted connection from the connections list
                                                    set_connections.update(|conns| {
                                                        conns.retain(|c| c.id != deleted_id);
                                                    });
                                                })
                                            />
                                        }
                                    }}
                                </For>
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            {move || show_connection.get().then(|| view! {
                <ConnectionModal
                    connection_name=connection_name
                    show_name_error=show_name_error
                    on_name_change=Callback::new(move |new_name| {
                        set_show_name_error.set(false);
                        set_connection_name.set(new_name);
                    })
                    on_cancel=Callback::new(move |_| {
                        set_show_name_error.set(false);
                        set_show_connection.set(false);
                    })
                    on_submit=Callback::new(move |_| {
                        if connection_name.get().trim().is_empty() {
                            set_show_name_error.set(true);
                        } else {
                            // Check if we have a link ID in the URL
                            if let Some(link_id) = connection_utils::get_link_id_from_url() {
                                // Join existing connection
                                join_connection(link_id);
                            } else {
                                // Create new connection
                                create_connection();
                            }
                        }
                    })
                />
            })}
        </div>
    }
}

fn get_connection_name(connection_id: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    
    // Try to get the name from localStorage
    let name_key = format!("conn-name-{}", connection_id);
    storage.get_item(&name_key).ok()?
}

pub fn get_stored_player_id() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item("player-id").ok()?
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_modal_starts_in_add_mode() {
        mount_to_body(|| view! { <FriendsConnect /> });
        
        // Open modal
        let new_conn_button = document()
            .query_selector("button")
            .unwrap()
            .expect("Should find New Connection button");
        new_conn_button.dispatch_event(&web_sys::Event::new("click").unwrap()).unwrap();
        
        // Wait for modal
        let _ = gloo_timers::future::TimeoutFuture::new(100).await;
        
        // Check label text indicates Add mode
        let label = document()
            .query_selector("label")
            .unwrap()
            .expect("Should find label");
            
        assert_eq!(label.text_content().unwrap(), "Connect to:");
    }


    #[wasm_bindgen_test]
    async fn test_empty_connection_name_shows_error() {
        mount_to_body(|| view! { <FriendsConnect /> });
        
        // Open modal
        let new_conn_button = document()
            .query_selector("button")
            .unwrap()
            .expect("Should find New Connection button");
        new_conn_button.dispatch_event(&web_sys::Event::new("click").unwrap()).unwrap();
        
        // Wait for modal
        let _ = gloo_timers::future::TimeoutFuture::new(100).await;
        
        // Find and click OK button with empty input
        let ok_button = document()
            .query_selector(".flex.justify-end.gap-4 button:last-child")
            .unwrap()
            .expect("Should find OK button");
            
        ok_button.dispatch_event(&web_sys::Event::new("click").unwrap()).unwrap();
        
        // Wait for error to appear
        let _ = gloo_timers::future::TimeoutFuture::new(100).await;
        
        // Check for error message
        let error = document()
            .query_selector("[data-test-id='connection-name-error']")  // We'll add this class to error message
            .unwrap()
            .expect("Should find error message");
            
        assert_eq!(
            error.text_content().unwrap(),
            "Please enter a name for your connection."
        );
    }

    #[wasm_bindgen_test]
    async fn test_cancel_button_closes_modal() {
        mount_to_body(|| view! { <FriendsConnect /> });
        
        // Open the modal first
        let new_conn_button = document()
            .query_selector("button")
            .unwrap()
            .expect("Should find New Connection button");
        new_conn_button.dispatch_event(&web_sys::Event::new("click").unwrap()).unwrap();
        
        // Wait for modal to appear
        let _ = gloo_timers::future::TimeoutFuture::new(100).await;
        
        // Find and click the cancel button
        let cancel_button = document()
            .query_selector("button.bg-gray-700")
            .unwrap()
            .expect("Should find cancel button");
        
        assert_eq!(cancel_button.text_content().unwrap(), "Cancel");
        
        cancel_button.dispatch_event(&web_sys::Event::new("click").unwrap()).unwrap();
        
        // Wait for modal to disappear
        let _ = gloo_timers::future::TimeoutFuture::new(100).await;
        
        // Verify modal is gone
        let modal = document().query_selector(".fixed").unwrap();
        assert!(modal.is_none(), "Modal should be closed after clicking cancel");
    }

    #[wasm_bindgen_test]
    async fn test_connection_name_input() {
        mount_to_body(|| view! { <FriendsConnect /> });
        
        // Open modal
        let button = document()
            .query_selector("button")
            .unwrap()
            .expect("Should find New Connection button");
        button.dispatch_event(&web_sys::Event::new("click").unwrap()).unwrap();
        
        // Wait for modal to appear
        let _ = gloo_timers::future::TimeoutFuture::new(100).await;
        
        // Find input section
        let label = document()
            .query_selector("label")
            .unwrap()
            .expect("Should find input label");
        
        assert_eq!(label.text_content().unwrap(), "Connect to:");

        // Check input field
        let input = document()
            .query_selector("input")
            .unwrap()
            .expect("Should find input field");
            
        assert_eq!(input.get_attribute("type").unwrap(), "text");
    }

    #[wasm_bindgen_test]
    async fn test_modal_structure_and_styling() {
        mount_to_body(|| view! { <FriendsConnect /> });
        
        // Click button to show modal
        let button = document()
            .query_selector("button")
            .unwrap()
            .expect("Should find New Connection button");
        button.dispatch_event(&web_sys::Event::new("click").unwrap()).unwrap();
        let _ = gloo_timers::future::TimeoutFuture::new(500).await;
        
        // Check modal structure
        let modal = document()
            .query_selector(".fixed")
            .unwrap()
            .expect("Modal should be present");
        
        // Should have proper z-index
        assert!(modal.class_list().contains("z-50"));
        
        // Inner modal content should have proper styling
        let content = modal
            .query_selector("div")
            .unwrap()
            .expect("Should find modal content div");
    }

    #[wasm_bindgen_test]
    async fn test_new_connection_button_shows_modal() {
        mount_to_body(|| view! { <FriendsConnect /> });
        
        // Find and click the New Connection button
        let button = document()
            .query_selector("button")
            .unwrap()
            .expect("Should find New Connection button");
            
        assert_eq!(button.text_content().unwrap(), "New Connection");
        
        // Initially modal should not be present
        let initial_modal = document().query_selector(".fixed").unwrap();
        assert!(initial_modal.is_none());
        
        // Click the button
        button.dispatch_event(&web_sys::Event::new("click").unwrap()).unwrap();
        let _ = gloo_timers::future::TimeoutFuture::new(500).await;
        
        // Now modal should be present
        let modal = document()
            .query_selector(".fixed")
            .unwrap()
            .expect("Modal should appear after click");
            
        let modal_title = modal
            .query_selector("h3")
            .unwrap()
            .expect("Modal should have title");
            
        assert_eq!(modal_title.text_content().unwrap(), "Make a connection!");
    }

    #[wasm_bindgen_test]
    async fn test_friends_connect_initializes_player_id() {
        // First ensure no player-id exists
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();
        storage.remove_item("player-id").unwrap();
        
        // Mount the component
        mount_to_body(|| view! { <FriendsConnect /> });

        let _ = gloo_timers::future::TimeoutFuture::new(1500).await;
        
        // After mounting, we should have a player-id in localStorage
        let player_id = storage.get_item("player-id").unwrap().unwrap();
        
        // Verify it's a valid UUID
        assert!(uuid::Uuid::parse_str(&player_id).is_ok());
    }

    #[wasm_bindgen_test]
    fn test_get_stored_player_id() {
        // First ensure no player-id exists
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();
        storage.remove_item("player-id").unwrap();
        
        // Initial check should return None
        assert!(get_stored_player_id().is_none());
        
        // Set a player-id
        storage.set_item("player-id", "test-123").unwrap();
        
        // Now we should get that value back
        assert_eq!(get_stored_player_id().unwrap(), "test-123");
    }

    #[wasm_bindgen_test]
    fn test_friends_connect_renders() {
        mount_to_body(|| view! { <FriendsConnect /> });
        
        let container = document()
            .query_selector("#friends-connect-container")
            .unwrap()
            .unwrap();
            
        let heading = container.query_selector("h2").unwrap().unwrap();
        assert_eq!(heading.text_content().unwrap(), "Connect with Friends");
    }

}