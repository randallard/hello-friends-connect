// src/connect_component.rs
use leptos::*;
use wasm_bindgen_test::*;
use serde::{Serialize, Deserialize};
use web_sys::Storage;

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

wasm_bindgen_test_configure!(run_in_browser);

#[component]
pub fn FriendsConnect(
    #[prop(default = "http://localhost:8080")] api_base: &'static str,
) -> impl IntoView {
    view! {
        <div id="friends-connect-container" class="max-w-md mx-auto p-4">
            <h2 class="text-xl font-bold mb-4">"Connect with Friends"</h2>
        </div>
    }
}

async fn create_new_connection(api_base: &str, player_id: &str) -> Result<Connection, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/connections", api_base))
        .json(&serde_json::json!({ "player_id": player_id }))
        .send()
        .await?;
        
    response.json::<Connection>().await
}

pub fn get_stored_player_id() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item("player-id").ok()?
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

#[wasm_bindgen_test]
async fn test_create_new_connection_api() {
    web_sys::console::log_1(&"Starting test".into());
    
    let result = create_new_connection(
        "http://64.181.233.1/friends",  // Note the /friends path
        "test-player"
    ).await;
    
    match &result {
        Ok(conn) => {
            web_sys::console::log_1(&format!("Success: {:?}", conn).into());
        }
        Err(e) => {
            web_sys::console::log_1(&format!("Error: {:?}", e).into());
        }
    }
    
    assert!(result.is_ok());
    let conn = result.unwrap();
    assert!(!conn.id.is_empty());
    assert!(!conn.link_id.is_empty());
    assert_eq!(conn.players.len(), 1);
    assert_eq!(conn.players[0], "test-player");
}