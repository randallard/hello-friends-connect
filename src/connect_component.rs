use leptos::*;
use leptos::prelude::*;  
use serde::{Serialize, Deserialize};
use wasm_bindgen::{JsValue,JsCast};
use std::ops::Not;
use wasm_bindgen_futures::spawn_local;
use web_sys::{console, WebSocket};

use crate::connection_modal::ConnectionModal; 
use crate::connection_utils;
use crate::connection_item::ConnectionItem;
use crate::notification_component::{NotificationList, start_notification_polling, add_notification};
use crate::websocket_client::{setup_websocket, join_connection_via_ws};

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

fn store_web_socket(id: &str, ws: WebSocket) {
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str(id),
            &ws,
        );
    }
}

fn get_web_socket(id: &str) -> Option<WebSocket> {
    web_sys::window().and_then(|window| {
        js_sys::Reflect::get(
            &window,
            &JsValue::from_str(id),
        ).ok()
        .and_then(|val| val.dyn_into::<WebSocket>().ok())
    })
}

#[component]
pub fn FriendsConnect() -> impl IntoView {
    let (show_connection, set_show_connection) = signal(false);
    let (connection_name, set_connection_name) = signal(String::new());
    let (show_name_error, set_show_name_error) = signal(false);
    let (api_error, set_api_error) = signal(String::new());
    let (current_connection, set_current_connection) = signal(None::<Connection>);

    let websocket_id = format!("ws_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    let (websocket_initialized, set_websocket_initialized) = create_signal(false);
    
    // Signal for active connections
    let (connections, set_connections) = signal(Vec::<Connection>::new());

    // Helper for logging
    let console_log = move |msg: &str| {
        console::log_1(&wasm_bindgen::JsValue::from_str(msg));
    };

    // Create a callback for when a connection becomes active
    let connections_setter = set_connections.clone();
    let current_connection_getter = current_connection.clone();
    let current_connection_setter = set_current_connection.clone();
    let on_connection_active = Callback::new(move |connection_id: String| {
        console_log(&format!("Connection is now active: {}", connection_id));

        // Update the connection status in our list
        connections_setter.update(|conns| {
            let mut updated = false;
            for conn in conns.iter_mut() {
                if conn.id == connection_id {
                    console_log(&format!("Updating connection status to Active for {}", connection_id));
                    conn.status = ConnectionStatus::Active;
                    updated = true;
                    break;
                }
            }
            if !updated {
                console_log(&format!("Warning: Connection {} not found in list", connection_id));
            }
        });
        
        current_connection_setter.update(|curr_conn| {
            if let Some(conn) = curr_conn {
                if conn.id == connection_id {
                    let mut updated_conn = conn.clone();
                    updated_conn.status = ConnectionStatus::Active;
                    *curr_conn = Some(updated_conn);
                }
            }
        });
        
        // Play a notification sound to alert the user
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Ok(audio) = web_sys::HtmlAudioElement::new_with_src("data:audio/wav;base64,UklGRl9vT19XQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YU") {
                    let _ = audio.play();
                }
            }
        }
        
        // Add a notification about the connection becoming active
        crate::notification_component::add_notification(
            format!("Connection is now active! Both players are connected.")
        );
    });
    
    // Setup notification polling
    Effect::new(move |_| {
        if let Some(player_id) = get_stored_player_id() {
            console_log(&format!("Starting notification polling for player: {}", player_id));
            start_notification_polling(player_id);
        }
    });

    let websocket_id_clone1 = websocket_id.clone();

    // Setup a single WebSocket for the player
    Effect::new(move |_| {
        if let Some(player_id) = get_stored_player_id() {
            console_log(&format!("Setting up WebSocket for player: {}", player_id));            
            
            let on_connection_active_ws = on_connection_active.clone();
            
            // Create a callback for notifications
            let on_notification = Callback::new(move |message: String| {
                console_log(&format!("Notification received via WebSocket: {}", message));
                add_notification(message);
            });
            
            // Set up the WebSocket with both callbacks
            match setup_websocket(player_id, on_connection_active_ws, on_notification) {
                Ok(ws) => {
                    console_log("WebSocket successfully set up");
                    store_web_socket(&websocket_id_clone1, ws);
                    set_websocket_initialized.set(true);
                },
                Err(e) => {
                    console_log(&format!("Failed to set up WebSocket: {:?}", e));
                }
            }
        }
    });

    // Ensure a player ID exists
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

    // Check for link ID in URL
    Effect::new(move |_| {
        if let Some(link_id) = connection_utils::get_link_id_from_url() {
            console_log(&format!("Found link ID in URL: {}", link_id));
            
            // Auto-open the connection modal
            set_show_connection.set(true);
        }
    });
    
    let websocket_id_clone2 = websocket_id.clone();

    // Load saved connections
    Effect::new(move |_| {
        // Use the existing load_saved_connections function
        let saved_connections = connection_utils::load_saved_connections();

        let ws_id = websocket_id_clone2.clone();
        let is_initialized = websocket_initialized.get();
        
        if !saved_connections.is_empty() {
            console_log("Loading saved connections from local storage");
            
            // Convert saved connections to Connection objects and add to connections signal
            for saved_conn in saved_connections {
                if let (Some(id), Some(link_id), Some(created_at)) = (
                    saved_conn.get("id").and_then(|v| v.as_str()),
                    saved_conn.get("link_id").and_then(|v| v.as_str()),
                    saved_conn.get("created_at").and_then(|v| v.as_i64())
                ) {
                    // Get expires_at from saved connection or calculate it
                    let expires_at = saved_conn.get("expires_at")
                        .and_then(|v| v.as_i64())
                        .unwrap_or_else(|| created_at + 86400); // 24 hours from creation in seconds

                    // Convert expires_at to milliseconds for comparison
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
                            conns.push(connection.clone());
                            
                            if is_initialized {
                                if let Some(ws) = get_web_socket(&ws_id) {
                                    // Skip if connection is expired
                                    if connection.status != ConnectionStatus::Expired {
                                        let _ = join_connection_via_ws(&ws, &connection.id);
                                        console_log(&format!("Auto-subscribed to saved connection: {}", connection.id));
                                    }
                                }
                            }
                        }
                    });
                }
            }
        }
    });

    // Define the notification callback
    let on_notification = Callback::new(move |message: String| {
        console_log(&format!("Notification received: {}", message));
        add_notification(message);
    });

    let websocket_id_clone3 = websocket_id.clone();

    // Create new connection function
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
        
        // Get the WebSocket to use after creation
        let ws_id = websocket_id_clone3;
        let is_initialized = websocket_initialized.get();
        
        spawn_local(async move {
            match connection_utils::create_connection(&player_id).await {
                Ok(mut connection) => {
                    console_log(&format!("Connection created with ID: {} and link_id: {}", 
                        connection.id, connection.link_id));
                    
                    // Subscribe to the new connection via WebSocket
                    if is_initialized {
                        if let Some(ws) = get_web_socket(&ws_id) {
                            match join_connection_via_ws(&ws, &connection.id) {
                                Ok(_) => console_log(&format!("Subscribed to new connection {} via WebSocket", connection.id)),
                                Err(e) => console_log(&format!("Failed to subscribe to connection: {:?}", e))
                            }
                        }
                    }
                    
                    // Check if we already have multiple players
                    if connection.players.len() >= 2 {
                        console_log("Two players are connected, setting status to Active");
                        connection.status = ConnectionStatus::Active;
                    }
                    
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
                    
                    // Add to connections list or update existing
                    set_connections.update(|conns| {
                        // Check if we already have this connection
                        let existing_index = conns.iter().position(|c| c.id == connection.id);
                        if let Some(index) = existing_index {
                            // Update existing connection
                            conns[index] = connection;
                        } else {
                            // Add new connection
                            conns.push(connection);
                        }
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

    let websocket_id_clone4 = websocket_id.clone();

    // Join existing connection function
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
        
        // Get WebSocket to use after joining
        let ws_id = websocket_id_clone4;
        let is_initialized = websocket_initialized.get();
        
        spawn_local(async move {
            match connection_utils::join_connection(&link_id, &player_id).await {
                Ok(mut connection) => {
                    console_log(&format!("Connection joined with ID: {} and link_id: {}", 
                        connection.id, connection.link_id));
                    
                    // Subscribe to this connection via WebSocket
                    if is_initialized {
                        if let Some(ws) = get_web_socket(&ws_id) {
                            match join_connection_via_ws(&ws, &connection.id) {
                                Ok(_) => console_log(&format!("Subscribed to joined connection {} via WebSocket", connection.id)),
                                Err(e) => console_log(&format!("Failed to subscribe to joined connection via WebSocket: {:?}", e))
                            }
                        }
                    }
                    
                    // Check if we already have multiple players
                    if connection.players.len() >= 2 {
                        console_log("Two players are connected, setting status to Active");
                        connection.status = ConnectionStatus::Active;
                    }
                    
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
                    
                    // Add to connections list or update existing
                    set_connections.update(|conns| {
                        // Check if we already have this connection
                        let existing_index = conns.iter().position(|c| c.id == connection.id);
                        if let Some(index) = existing_index {
                            // Update existing connection
                            conns[index] = connection;
                        } else {
                            // Add new connection
                            conns.push(connection);
                        }
                    });
                    
                    // Close the modal
                    set_show_connection.set(false);
                    set_connection_name.set(String::new());
                },
                Err(e) => {
                    // Error handling remains the same
                    let error_msg = e.as_string().unwrap_or_else(|| format!("{:?}", e));
                    console_log(&format!("Error joining connection: {}", error_msg));
                    
                    // Handle special error cases
                    if error_msg.contains("Connection already has maximum players") || 
                       error_msg.contains("Player already in connection") {
                        
                        // Set appropriate error message
                        let message = if error_msg.contains("Connection already has maximum players") {
                            format!(
                                "This connection is already full. Creating a new one for {}.", 
                                name_clone
                            )
                        } else {
                            format!(
                                "You're already connected! Creating a new connection for {}.", 
                                name_clone
                            )
                        };
                        set_api_error.set(message);
                        
                        // Clear the URL
                        if let Some(window) = web_sys::window() {
                            if let Ok(history) = window.history() {
                                let _ = history.push_state_with_url(
                                    &wasm_bindgen::JsValue::NULL, 
                                    "", 
                                    Some(window.location().pathname().unwrap_or_default().as_str())
                                );
                            }
                        }
                        
                        // Create a new connection instead
                        let player_id_clone = player_id.clone();
                        let name_clone2 = name_clone.clone();
                        let ws_id2 = ws_id.clone();
                        let is_initialized2 = is_initialized;
                        
                        spawn_local(async move {
                            match connection_utils::create_connection(&player_id_clone).await {
                                Ok(connection) => {
                                    console_log(&format!("Auto-created new connection with ID: {}", connection.id));
                                    
                                    // Subscribe to this new connection
                                    if is_initialized2 {
                                        if let Some(ws) = get_web_socket(&ws_id2) {
                                            match join_connection_via_ws(&ws, &connection.id) {
                                                Ok(_) => console_log(&format!("Subscribed to auto-created connection {} via WebSocket", connection.id)),
                                                Err(e) => console_log(&format!("Failed to subscribe to auto-created connection: {:?}", e))
                                            }
                                        }
                                    }
                                    
                                    // Save friendly name for this connection
                                    if let Some(window) = web_sys::window() {
                                        if let Ok(Some(storage)) = window.local_storage() {
                                            let _ = storage.set_item(&format!("conn-name-{}", connection.id), &name_clone2);
                                        }
                                    }
                                    
                                    // Save the connection to localStorage
                                    let _ = connection_utils::save_connection_to_local_storage(&connection, &name_clone2);
                                    
                                    // Update signals
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
                                    let error_msg = format!("Error creating new connection: {:?}", e);
                                    console_log(&error_msg);
                                    set_api_error.set(error_msg);
                                }
                            }
                        });
                    }
                    else {
                        // For all other errors, simply display them
                        set_api_error.set(format!("Error joining connection: {}", error_msg));
                    }
                }
            }
        });
    };
// Fix the View macro with all Fn trait issues resolved
view! {
    <div id="friends-connect-container" class="max-w-md mx-auto p-4 bg-gray-900 text-gray-100">
        <h2 class="text-xl font-bold mb-4 text-gray-100">"Connect with Friends"</h2>
        
        // API error message
        <Show
            when=move || !api_error.get().is_empty()
            fallback=|| view! { <></> }
        >
            <div class="bg-red-900 text-red-100 p-4 rounded mb-4">
                {move || api_error.get()}
            </div>
        </Show>
        
        // New Connection button
        <button
            class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 rounded text-gray-100 mb-4"
            on:click=move |_| {
                set_show_name_error.update(|v| *v = false);
                set_show_connection.update(|v| *v = true);
            }
        >
            "New Connection"
        </button>

        // Display connections list
        <div class="mt-4">
            <Show
                when=move || !connections.get().is_empty()
                fallback=|| view! {
                    <div class="text-gray-400 text-sm mt-2">
                        "No connections yet. Click 'New Connection' to create one."
                    </div>
                }
            >
                <div class="border border-gray-700 rounded overflow-hidden">
                    <For
                        each=move || connections.get()
                        key=|conn| conn.id.clone()
                        children=move |connection: Connection| {
                            let conn_id = connection.id.clone();
                            let name = get_connection_name(&conn_id).unwrap_or_else(|| "Unnamed Connection".to_string());
                            
                            let set_connections_clone = set_connections.clone();
                            
                            view! {
                                <ConnectionItem 
                                    connection=connection.clone() 
                                    name=name
                                    on_delete=Callback::new(move |deleted_id: String| {
                                        set_connections_clone.update(|conns| {
                                            conns.retain(|c| c.id != deleted_id);
                                        });
                                    })
                                />
                            }
                        }
                    />
                </div>
            </Show>
        </div>

        // Connection modal
        <Show
            when=move || show_connection.get()
            fallback=|| view! { <></> }
        >
            <ConnectionModal
                connection_name=connection_name.clone()
                show_name_error=show_name_error.clone()
                on_name_change=Callback::new(move |new_name: String| {
                    set_show_name_error.update(|v| *v = false);
                    set_connection_name.update(|v| *v = new_name);
                })
                on_cancel=Callback::new(move |_| {
                    set_show_name_error.update(|v| *v = false);
                    set_show_connection.update(|v| *v = false);
                })
                on_submit=Callback::new(move |existing_connection: Option<Connection>| {
                    let name_val = connection_name.get();
                    if name_val.trim().is_empty() {
                        set_show_name_error.update(|v| *v = true);
                    } else {
                        // Check if we have a link ID in the URL
                        let url_link_id = connection_utils::get_link_id_from_url();
                        if url_link_id.is_some() && api_error.get().is_empty() {
                            // Join existing connection using a local definition
                            let link_id = url_link_id.unwrap();
                            let player_id = get_stored_player_id().unwrap_or_else(|| {
                                let new_id = uuid::Uuid::new_v4().to_string();
                                if let Some(window) = web_sys::window() {
                                    if let Ok(Some(storage)) = window.local_storage() {
                                        let _ = storage.set_item("player-id", &new_id);
                                    }
                                }
                                new_id
                            });
                    
                            let name_clone = name_val.clone();
                            console_log(&format!("Joining connection with link ID: {}", link_id));
                            
                            // Reset error state
                            set_api_error.update(|v| *v = String::new());
                            
                            // Get WebSocket to use after joining
                            let ws_id = format!("ws_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
                            let is_initialized = websocket_initialized.get();
                            
                            spawn_local(async move {
                                match connection_utils::join_connection(&link_id, &player_id).await {
                                    Ok(mut connection) => {
                                        console_log(&format!("Connection joined with ID: {} and link_id: {}", 
                                            connection.id, connection.link_id));
                                        
                                        // Subscribe to this connection via WebSocket
                                        if is_initialized {
                                            if let Some(ws) = get_web_socket(&ws_id) {
                                                match join_connection_via_ws(&ws, &connection.id) {
                                                    Ok(_) => console_log(&format!("Subscribed to joined connection {} via WebSocket", connection.id)),
                                                    Err(e) => console_log(&format!("Failed to subscribe to joined connection via WebSocket: {:?}", e))
                                                }
                                            }
                                        }
                                        
                                        // Check if we already have multiple players
                                        if connection.players.len() >= 2 {
                                            console_log("Two players are connected, setting status to Active");
                                            connection.status = ConnectionStatus::Active;
                                        }
                                        
                                        // Save friendly name for this connection
                                        if let Some(window) = web_sys::window() {
                                            if let Ok(Some(storage)) = window.local_storage() {
                                                let _ = storage.set_item(&format!("conn-name-{}", connection.id), &name_clone);
                                            }
                                        }
                                        
                                        // Save the connection for later
                                        let _ = connection_utils::save_connection_to_local_storage(&connection, &name_clone);
                                        
                                        // Update current connection
                                        set_current_connection.update(|curr| *curr = Some(connection.clone()));
                                        
                                        // Add to connections list or update existing
                                        set_connections.update(|conns| {
                                            // Check if we already have this connection
                                            let existing_index = conns.iter().position(|c| c.id == connection.id);
                                            if let Some(index) = existing_index {
                                                // Update existing connection
                                                conns[index] = connection;
                                            } else {
                                                // Add new connection
                                                conns.push(connection);
                                            }
                                        });
                                        
                                        // Close the modal
                                        set_show_connection.update(|v| *v = false);
                                        set_connection_name.update(|v| *v = String::new());
                                    },
                                    Err(e) => {
                                        // Handle error
                                        let error_msg = e.as_string().unwrap_or_else(|| format!("{:?}", e));
                                        console_log(&format!("Error joining connection: {}", error_msg));
                                        
                                        // Set error message
                                        set_api_error.update(|v| *v = format!("Error joining connection: {}", error_msg));
                                    }
                                }
                            });
                        } else if let Some(connection) = existing_connection {
                            // Use the already created connection
                            console_log(&format!("Using pre-created connection: {}", connection.id));
                            
                            // Save friendly name for this connection
                            if let Some(window) = web_sys::window() {
                                if let Ok(Some(storage)) = window.local_storage() {
                                    let _ = storage.set_item(&format!("conn-name-{}", connection.id), &name_val);
                                }
                            }
                            
                            // Use a newly generated ID for WebSocket to avoid moved value
                            let ws_id = format!("ws_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
                            let is_ws_initialized = websocket_initialized.get();
                            
                            // Subscribe to the connection via WebSocket
                            if is_ws_initialized {
                                if let Some(ws) = get_web_socket(&ws_id) {
                                    let conn_id = connection.id.clone();
                                    match join_connection_via_ws(&ws, &conn_id) {
                                        Ok(_) => console_log(&format!("Subscribed to pre-created connection {} via WebSocket", conn_id)),
                                        Err(e) => console_log(&format!("Failed to subscribe to pre-created connection: {:?}", e))
                                    }
                                }
                            }
                            
                            // Save the connection for later
                            let connection_to_save = connection.clone();
                            let _ = connection_utils::save_connection_to_local_storage(&connection_to_save, &name_val);
                            
                            // Update current connection
                            set_current_connection.set(Some(connection.clone()));
                            
                            // Add to connections list
                            let connection_to_add = connection.clone();
                            set_connections.update(|conns| {
                                conns.push(connection_to_add);
                            });
                            
                            // Close the modal
                            set_show_connection.update(|v| *v = false);
                            set_connection_name.update(|v| *v = String::new());
                        } else {
                            // Create a connection directly in the callback
                            let name_clone = name_val.clone();
                            
                            let player_id = get_stored_player_id().unwrap_or_else(|| {
                                let new_id = uuid::Uuid::new_v4().to_string();
                                if let Some(window) = web_sys::window() {
                                    if let Ok(Some(storage)) = window.local_storage() {
                                        let _ = storage.set_item("player-id", &new_id);
                                    }
                                }
                                new_id
                            });
                            
                            console_log(&format!("Creating connection with name: {}", name_val));
                            
                            // Reset error state
                            set_api_error.update(|v| *v = String::new());
                            
                            // Use a newly generated ID for WebSocket to avoid moved value
                            let ws_id = format!("ws_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
                            let is_initialized = websocket_initialized.get();
                            
                            spawn_local(async move {
                                match connection_utils::create_connection(&player_id).await {
                                    Ok(mut connection) => {
                                        console_log(&format!("Connection created with ID: {} and link_id: {}", 
                                            connection.id, connection.link_id));
                                        
                                        // Subscribe to the new connection via WebSocket
                                        if is_initialized {
                                            if let Some(ws) = get_web_socket(&ws_id) {
                                                match join_connection_via_ws(&ws, &connection.id) {
                                                    Ok(_) => console_log(&format!("Subscribed to new connection {} via WebSocket", connection.id)),
                                                    Err(e) => console_log(&format!("Failed to subscribe to connection: {:?}", e))
                                                }
                                            }
                                        }
                                        
                                        // Check if we already have multiple players
                                        if connection.players.len() >= 2 {
                                            console_log("Two players are connected, setting status to Active");
                                            connection.status = ConnectionStatus::Active;
                                        }
                                        
                                        // Save friendly name for this connection
                                        if let Some(window) = web_sys::window() {
                                            if let Ok(Some(storage)) = window.local_storage() {
                                                let _ = storage.set_item(&format!("conn-name-{}", connection.id), &name_clone);
                                            }
                                        }
                                        
                                        // Save the connection for later
                                        let _ = connection_utils::save_connection_to_local_storage(&connection, &name_clone);
                                        
                                        // Update current connection
                                        set_current_connection.update(|curr| *curr = Some(connection.clone()));
                                        
                                        // Add to connections list or update existing
                                        set_connections.update(|conns| {
                                            // Check if we already have this connection
                                            let existing_index = conns.iter().position(|c| c.id == connection.id);
                                            if let Some(index) = existing_index {
                                                // Update existing connection
                                                conns[index] = connection;
                                            } else {
                                                // Add new connection
                                                conns.push(connection);
                                            }
                                        });
                                        
                                        // Close the modal
                                        set_show_connection.update(|v| *v = false);
                                        set_connection_name.update(|v| *v = String::new());
                                    },
                                    Err(e) => {
                                        let error_msg = format!("Error creating connection: {:?}", e);
                                        console_log(&error_msg);
                                        set_api_error.update(|v| *v = error_msg);
                                    }
                                }
                            });
                        }
                    }
                })
            />
        </Show>
        <NotificationList />
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
