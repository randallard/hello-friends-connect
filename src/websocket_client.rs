// In websocket_client.rs - modify to support single WebSocket for all connections

use std::cell::RefCell;

use leptos::reactive::wrappers::write::SignalSetter;
use web_sys::{WebSocket, MessageEvent, ErrorEvent, CloseEvent};
use wasm_bindgen::{prelude::*, JsCast};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use leptos::*;
use leptos::prelude::{Callable, Callback, RwSignal, Set, WriteSignal};
use crate::config::get_config;

thread_local! {
    static NOTIFICATIONS: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

#[derive(Deserialize, Serialize, Debug)]
struct WsMessage {
    event_type: String,
    payload: serde_json::Value,
}

// Used to track heartbeat state
#[derive(Clone, Debug)]
struct HeartbeatState {
    last_ack_timestamp: Option<i64>,
    missed_count: u8,
}

fn show_notification(message: String) {
    if let Some(window) = web_sys::window() {
        // First try to use the browser's Notification API if available
        if let Ok(notification_class) = js_sys::Reflect::get(&window, &JsValue::from_str("Notification")) {
            // Check permission
            if let Ok(permission) = js_sys::Reflect::get(&notification_class, &JsValue::from_str("permission")) {
                if permission.as_string().unwrap_or_default() == "granted" {
                    // Create notification
                    let _ = web_sys::Notification::new_with_options(
                        &message,
                        web_sys::NotificationOptions::new().body("You have a new notification")
                    );
                    return;
                }
            }
        }
        
        // Fall back to showing an alert or custom UI element
        let _ = window.alert_with_message(&format!("New notification: {}", message));
    }
}

pub fn setup_websocket(
    player_id: String, 
    on_connection_active: Callback<String>,
    on_notification: Callback<String>,
    connection_status: RwSignal<bool>,
    websocket_id: String,
) -> Result<WebSocket, JsValue> {

    // Create WebSocket connection
    let api_url = get_config().api_base_url.clone();
    
    // Get the domain by removing 'http://' or 'https://' and any path after the domain
    let modified_url = api_url.replace("http://", "").replace("https://", "");
    // Then split it
    let domain = modified_url.split('/').next().unwrap_or("64.181.233.1");
    
    let ws_url = format!("ws://{}/ws?player_id={}", domain, player_id);
    let ws = WebSocket::new(&ws_url)?;
    
    // Log for debugging
    let console_log = move |msg: &str| {
        web_sys::console::log_1(&JsValue::from_str(msg));
    };
    
    // Setup message handler
    let on_message_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            let message = String::from(txt);
            if !message.contains("heartbeat_ack") {
                console_log(&format!("WebSocket message received: {}", message));
            }
            // Try to parse the message as JSON
            if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&message) {
                match ws_msg.event_type.as_str() {
                    "connection_status_updated" | "status_update" | "connection_updated" | "connection_status_update"=> {
                        if let Some(status) = ws_msg.payload.get("status").and_then(|s| s.as_str()) {
                            if status == "Active" {
                                if let Some(conn_id) = ws_msg.payload.get("connection_id").and_then(|id| id.as_str()) {
                                    console_log(&format!("Connection {} is now active!", conn_id));
                                    // Call the callback with the connection ID
                                    on_connection_active.run(conn_id.to_string());
                                }
                            }
                        }
                    },
                    "notification" => {
                        // Handle incoming notification
                        if let Some(notification_msg) = ws_msg.payload.get("message").and_then(|m| m.as_str()) {
                            console_log(&format!("Notification received: {}", notification_msg));
                            on_notification.run(notification_msg.to_string());
                            
                            // Check if this is a "player joined" notification
                            if notification_msg.contains("joined your connection") {
                                if let Some(conn_id) = ws_msg.payload.get("connection_id").and_then(|id| id.as_str()) {
                                    console_log(&format!("Player joined connection {}, marking as active", conn_id));
                                    on_connection_active.run(conn_id.to_string());
                                }
                            }
                        }
                    },
                    "new_message" => {
                        // Handle new message
                        if let (Some(conn_id), Some(sender_id), Some(content)) = (
                            ws_msg.payload.get("connection_id").and_then(|id| id.as_str()),
                            ws_msg.payload.get("sender_id").and_then(|id| id.as_str()),
                            ws_msg.payload.get("content").and_then(|c| c.as_str()),
                        ) {
                            console_log(&format!("New message in connection {}: {}", conn_id, content));
                            let notification_msg = format!("Message from {}: {}", sender_id, content);
                            on_notification.run(notification_msg);
                        }
                    },
                    "welcome" => {
                        // Handle welcome message
                        console_log("WebSocket connection established successfully");
                    },
                    "heartbeat_ack" => {
                        // Reset missed heartbeat counter since we received an ack
                        if let Some(timestamp) = ws_msg.payload.get("timestamp").and_then(|t| t.as_i64()) {
                            // We could store this timestamp if needed for monitoring
                            
                            // Reset missed heartbeats in our tracking
                            if let Some(window) = web_sys::window() {
                                let _ = js_sys::Reflect::set(
                                    &window,
                                    &JsValue::from_str("missed_heartbeats"),
                                    &JsValue::from_f64(0.0),
                                );
                            }
                        }
                    },
                    "join_connection_ack" => {
                        // Handle join connection acknowledgment
                        if let Some(conn_id) = ws_msg.payload.get("connection_id").and_then(|id| id.as_str()) {
                            console_log(&format!("Successfully subscribed to connection {}", conn_id));
                        }
                    },
                    "connections_list" => {
                        // Handle list of connections this client is subscribed to
                        if let Some(connections) = ws_msg.payload.get("connections").and_then(|c| c.as_array()) {
                            let conn_ids: Vec<String> = connections.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect();
                                
                            console_log(&format!("Subscribed to connections: {:?}", conn_ids));
                        }
                    },
                    "error" => {
                        // Handle error messages
                        if let Some(error) = ws_msg.payload.get("error").and_then(|e| e.as_str()) {
                            console_log(&format!("WebSocket error: {}", error));
                        }
                    },
                    _ => console_log(&format!("Unhandled event type: {}", ws_msg.event_type)),
                }
            } else {
                console_log(&format!("Failed to parse WebSocket message: {}", message));
            }
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    
    ws.set_onmessage(Some(on_message_callback.as_ref().unchecked_ref()));
    on_message_callback.forget();
    
    // Setup open handler
    let ws_clone = ws.clone();
    let open_callback = Closure::wrap(Box::new(move |_| {
        console_log("WebSocket connection established");
        
        // Set up a heartbeat to keep the connection alive
        let ws_heartbeat = ws_clone.clone();

        // Reset heartbeat counter to zero when connection is established
        if let Some(window) = web_sys::window() {
            let _ = js_sys::Reflect::set(
                &window,
                &JsValue::from_str("missed_heartbeats"),
                &JsValue::from_f64(0.0),
            );
        }
        
        // Send an initial message to verify the connection works
        if ws_clone.ready_state() == WebSocket::OPEN {
            let initial_message = WsMessage {
                event_type: "connection_check".to_string(),
                payload: serde_json::json!({}),
            };
            
            if let Ok(message_json) = serde_json::to_string(&initial_message) {
                if let Err(e) = ws_clone.send_with_str(&message_json) {
                    console_log(&format!("Failed to send initial connection check message: {:?}", e));
                } else {
                    console_log("Sent initial connection check message");
                }
            }
        }

        spawn_local(async move {
            loop {
                // Send a heartbeat every 30 seconds
                gloo_timers::future::TimeoutFuture::new(30000).await;


                let mut missed_count = 0;
                if let Some(window) = web_sys::window() {
                    if let Ok(count) = js_sys::Reflect::get(
                        &window,
                        &JsValue::from_str("missed_heartbeats"),
                    ) {
                        missed_count = count.as_f64().unwrap_or(0.0) as u8;
                        
                        // Increment the counter
                        let _ = js_sys::Reflect::set(
                            &window,
                            &JsValue::from_str("missed_heartbeats"),
                            &JsValue::from_f64((missed_count + 1) as f64),
                        );
                        
                        // Log if we've missed too many
                        if missed_count >= 2 {
                            console_log(&format!("Warning: {} consecutive heartbeats missed", missed_count));
                        }
                    }
                }
                
                if ws_heartbeat.ready_state() == WebSocket::OPEN {
                    let heartbeat = WsMessage {
                        event_type: "heartbeat".to_string(),
                        payload: serde_json::json!({}),
                    };
                    
                    if let Ok(heartbeat_json) = serde_json::to_string(&heartbeat) {
                        let _ = ws_heartbeat.send_with_str(&heartbeat_json);
                        // Only log this once when we start the heartbeat process
                        static FIRST_HEARTBEAT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
                        if FIRST_HEARTBEAT.swap(false, std::sync::atomic::Ordering::SeqCst) {
                            console_log("WebSocket heartbeat monitoring started");
                        }
                    }
                } else {
                    // Connection is closed, stop the heartbeat
                    console_log("WebSocket connection closed, stopping heartbeat");
                    break;
                }
            }
        });
    }) as Box<dyn FnMut(JsValue)>);
    
    ws.set_onopen(Some(open_callback.as_ref().unchecked_ref()));
    open_callback.forget();
    
    let player_id_clone = player_id.clone();
    let websocket_id_clone = websocket_id.clone();
    let on_connection_active_clone = on_connection_active.clone();
    let on_notification_clone = on_notification.clone();
    let connection_status_clone = connection_status;

    // Setup error handler
    let error_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
        console_log(&format!("WebSocket error: {:?}", e));
        // Ensure connection status is set to false on error
        connection_status.set(false);
        
        // Attempt to reconnect
        reconnect_websocket(
            player_id_clone.clone(), 
            websocket_id_clone.clone(),
            on_connection_active_clone.clone(), 
            on_notification_clone.clone(),
            connection_status_clone
        );
    }) as Box<dyn FnMut(ErrorEvent)>);

    ws.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
    error_callback.forget();
    
    // Setup close handler
    let close_callback = Closure::wrap(Box::new(move |e: CloseEvent| {
        console_log(&format!("WebSocket closed: code={}, reason={}", e.code(), e.reason()));
    }) as Box<dyn FnMut(CloseEvent)>);
    
    ws.set_onclose(Some(close_callback.as_ref().unchecked_ref()));
    close_callback.forget();
    
    Ok(ws)
}

pub fn join_connection_via_ws(ws: &WebSocket, connection_id: &str) -> Result<(), JsValue> {
    // Send a join message
    let join_message = WsMessage {
        event_type: "join_connection".to_string(),
        payload: serde_json::json!({
            "connection_id": connection_id
        }),
    };
    
    let join_json = serde_json::to_string(&join_message).map_err(|e| {
        JsValue::from_str(&format!("Failed to serialize join message: {:?}", e))
    })?;
    
    ws.send_with_str(&join_json)?;
    web_sys::console::log_1(&JsValue::from_str(
        &format!("Sent join connection request for {}", connection_id)
    ));
    
    // Then send a specific status update message to ensure all clients are notified
    let status_message = WsMessage {
        event_type: "connection_status_update".to_string(),
        payload: serde_json::json!({
            "connection_id": connection_id,
            "status": "Active"
        }),
    };
    
    let status_json = serde_json::to_string(&status_message).map_err(|e| {
        JsValue::from_str(&format!("Failed to serialize status message: {:?}", e))
    })?;
    
    // Send after a short delay to ensure join is processed first
    let ws_clone = ws.clone();
    let connection_id_clone = connection_id.to_string();
    
    spawn_local(async move {
        // Wait a short time to ensure join is processed
        gloo_timers::future::TimeoutFuture::new(500).await;
        
        if ws_clone.ready_state() == WebSocket::OPEN {
            if let Err(e) = ws_clone.send_with_str(&status_json) {
                web_sys::console::log_1(&JsValue::from_str(
                    &format!("Failed to send status update for {}: {:?}", connection_id_clone, e)
                ));
            } else {
                web_sys::console::log_1(&JsValue::from_str(
                    &format!("Sent active status update for connection {}", connection_id_clone)
                ));
            }
        }
    });
    
    Ok(())
}
// Add to websocket_client.rs
pub fn store_web_socket(id: &str, ws: WebSocket) {
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str(id),
            &ws,
        );
    }
}

pub fn get_web_socket(id: &str) -> Option<WebSocket> {
    web_sys::window().and_then(|window| {
        js_sys::Reflect::get(
            &window,
            &JsValue::from_str(id),
        ).ok()
        .and_then(|val| val.dyn_into::<WebSocket>().ok())
    })
}
// Add to websocket_client.rs - could be added at the bottom
pub fn reconnect_websocket(
    player_id: String,
    websocket_id: String,
    on_connection_active: Callback<String>,
    on_notification: Callback<String>,
    connection_status: RwSignal<bool>
) {

    let websocket_id_clone = websocket_id.clone();

    let console_log = move |msg: &str| {
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(msg));
    };
    
    spawn_local(async move {
        console_log("Attempting to reconnect WebSocket...");
        
        // Wait a moment before reconnecting
        gloo_timers::future::TimeoutFuture::new(2000).await;
        
        match setup_websocket(player_id, on_connection_active, on_notification, connection_status,websocket_id) {
            Ok(ws) => {
                console_log("WebSocket reconnection successful");
                store_web_socket(&websocket_id_clone, ws);
            },
            Err(e) => {
                console_log(&format!("WebSocket reconnection failed: {:?}", e));
                // Schedule another reconnection attempt
                gloo_timers::future::TimeoutFuture::new(5000).await;
                // We'd implement the reconnection retry logic here
            }
        }
    });
}