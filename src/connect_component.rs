use leptos::*;
use leptos::prelude::*;  
use serde::{Serialize, Deserialize};

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
        || ()
    });

    view! {
        <div id="friends-connect-container" class="max-w-md mx-auto p-4">
            <h2 class="text-xl font-bold mb-4">"Connect with Friends"</h2>
            <button
                class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-white"
                on:click=move |_| {
                    set_show_name_error.set(false); 
                    set_show_connection.set(true)
                }
            >
                "New Connection"
            </button>

            {move || show_connection.get().then(|| view! {
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    <div class="bg-slate-800 p-6 rounded-lg shadow-xl max-w-md w-full mx-4 text-white">
                        <h3 class="text-xl font-bold mb-4">"Make a connection!"</h3>
                        <div class="flex flex-col gap-4">
                            <div>
                                <label class="block text-sm font-medium mb-1">
                                    "Connect to:"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 rounded bg-slate-700 border border-slate-600 text-white"
                                    prop:value=connection_name
                                    on:input=move |ev| {
                                        set_show_name_error.set(false);
                                        set_connection_name.set(event_target_value(&ev))
                                    }
                                />
                                {move || show_name_error.get().then(|| view! {
                                    <div class="mt-2 text-red-500 text-sm">
                                        "Please enter a name for your connection."
                                    </div>
                                })}
                            </div>


                            <div class="flex justify-end gap-4">
                                <button
                                    class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded"
                                    on:click=move |_| {
                                        set_show_name_error.set(false);
                                        set_show_connection.set(false)
                                    }
                                >
                                    "Cancel"
                                </button>
                                <button
                                    class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded"
                                    on:click=move |_| {
                                        if connection_name.get().trim().is_empty() {
                                            set_show_name_error.set(true);
                                        } else {
                                            set_show_name_error.set(false);
                                            set_show_connection.set(false);
                                            set_connection_name.set(String::new());
                                        }
                                    }
                                >
                                    "OK"
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
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
            .query_selector(".text-red-500.text-sm")  // We'll add this class to error message
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
        assert!(label.class_list().contains("block"));
        assert!(label.class_list().contains("text-sm"));
        assert!(label.class_list().contains("font-medium"));
        assert!(label.class_list().contains("mb-1"));
        
        // Check input field
        let input = document()
            .query_selector("input")
            .unwrap()
            .expect("Should find input field");
            
        assert_eq!(input.get_attribute("type").unwrap(), "text");
        let class_list = input.get_attribute("class").unwrap();
        assert!(class_list.contains("w-full"));
        assert!(class_list.contains("px-4"));
        assert!(class_list.contains("py-2"));
        assert!(class_list.contains("rounded"));
        assert!(class_list.contains("bg-slate-700"));
        assert!(class_list.contains("border"));
        assert!(class_list.contains("border-slate-600"));
        assert!(class_list.contains("text-white"));
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
            
        // Should have full screen overlay with dark background
        assert!(modal.class_list().contains("inset-0"));
        assert!(modal.class_list().contains("bg-black"));
        assert!(modal.class_list().contains("bg-opacity-50"));
        
        // Should have centered flex container
        assert!(modal.class_list().contains("flex"));
        assert!(modal.class_list().contains("items-center"));
        assert!(modal.class_list().contains("justify-center"));
        
        // Should have proper z-index
        assert!(modal.class_list().contains("z-50"));
        
        // Inner modal content should have proper styling
        let content = modal
            .query_selector("div")
            .unwrap()
            .expect("Should find modal content div");
            
        assert!(content.class_list().contains("bg-slate-800"));
        assert!(content.class_list().contains("p-6"));
        assert!(content.class_list().contains("rounded-lg"));
        assert!(content.class_list().contains("shadow-xl"));
        assert!(content.class_list().contains("max-w-md"));
        assert!(content.class_list().contains("w-full"));
        assert!(content.class_list().contains("mx-4"));
        assert!(content.class_list().contains("text-white"));
    }

    #[wasm_bindgen_test]
    fn test_new_connection_button_styling() {
        mount_to_body(|| view! { <FriendsConnect /> });
        
        let button = document()
            .query_selector("button")
            .unwrap()
            .expect("Should find New Connection button");
            
        // Check that button has the expected classes
        let class_list = button.get_attribute("class").unwrap();
        assert!(class_list.contains("px-4"));
        assert!(class_list.contains("py-2"));
        assert!(class_list.contains("bg-blue-600"));
        assert!(class_list.contains("hover:bg-blue-700"));
        assert!(class_list.contains("rounded"));
        assert!(class_list.contains("text-white"));
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