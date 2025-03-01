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

    let (show_connection, set_show_connection) = create_signal(false);
    let (connection_name, set_connection_name) = create_signal(String::new());
    let (show_name_error, set_show_name_error) = create_signal(false);

    create_effect(move |_| {
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
        <div id="friends-connect-container" class="max-w-md mx-auto p-4">
            <h2 class="text-xl font-bold mb-4">"Connect with Friends"</h2>
            <button
                class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-white"
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