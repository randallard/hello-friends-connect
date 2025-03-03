use leptos::*;
use leptos::prelude::*;  
use serde::{Serialize, Deserialize};

use crate::connection_modal::ConnectionModal; 

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

    Effect::new( move |_| {
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

    view! {
        <div id="friends-connect-container" class="max-w-md mx-auto p-4 bg-gray-900 text-gray-100">
            <h2 class="text-xl font-bold mb-4 text-gray-100">"Connect with Friends"</h2>
            <button
                class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 rounded text-gray-100"
                on:click=move |_| {
                    set_show_name_error.set(false); 
                    set_show_connection.set(true);
                }
            >
                "New Connection"
            </button>

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
                            set_show_name_error.set(false);
                            set_show_connection.set(false);
                            set_connection_name.set(String::new());
                        }
                    })
                />
            })}
        </div>
    }
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