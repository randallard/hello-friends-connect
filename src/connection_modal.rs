use leptos::*;
use leptos::prelude::*;
use web_sys::{window, UrlSearchParams, console};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::connection_utils::get_link_id_from_url;
use crate::connect_component::{Connection, get_stored_player_id};

#[component]
pub fn ConnectionModal(
    #[prop(into)] connection_name: Signal<String>,
    #[prop(into)] show_name_error: Signal<bool>,
    #[prop(into)] on_name_change: Callback<String>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_submit: Callback<Option<Connection>>,
    #[prop(optional)] is_view_mode: bool,
    #[prop(optional)] on_delete: Option<Callback<()>>,
    #[prop(optional)] connection_link_id: Option<String>,
) -> impl IntoView {
    // Create signals for the link ID
    let (link_id, set_link_id) = signal(String::new());
    let (loading_link, set_loading_link) = signal(false);
    let (link_error, set_link_error) = signal(String::new());
    let (created_connection, set_created_connection) = signal(None::<Connection>);
    
    // Function for console logging
    let console_log = move |msg: &str| {
        console::log_1(&wasm_bindgen::JsValue::from_str(msg));
    };
      
    // Update the request_new_link_id function to store the connection
    let request_new_link_id = move || {
        console_log("Generating new link ID via API call");
        // Check if we have a player ID
        if let Some(player_id) = get_stored_player_id() {
            // Set loading state
            set_loading_link.set(true);
            set_link_error.set(String::new());
            
            // Clone player ID for the async closure
            let player_id_clone = player_id.clone();
            
            // Spawn async task to request link ID
            spawn_local(async move {
                match crate::connection_utils::create_connection(&player_id_clone).await {
                    Ok(connection) => {
                        // Extract the link ID
                        let new_link_id = connection.link_id.clone();
                        console_log(&format!("Generated new link ID: {}", new_link_id));
                        
                        // Update the UI
                        set_link_id.set(new_link_id);
                        set_loading_link.set(false);
                        
                        // Store the created connection for later use
                        set_created_connection.set(Some(connection));
                    },
                    Err(e) => {
                        // Handle error, but don't block the form submission
                        let error_msg = format!("Failed to generate link: {:?}", e);
                        console_log(&error_msg);
                        set_link_error.set(error_msg);
                        set_loading_link.set(false);
                    }
                }
            });
        } else {
            // No player ID available
            set_link_error.set("No player ID found".to_string());
        }
    };
    
    // Check URL for link parameter or request a new link ID
    let initialize_link_id = move || {
        if let Some(custom_link_id) = connection_link_id.clone() {
            // Use the provided connection link ID when in view mode
            console_log(&format!("Modal using provided link ID: {}", custom_link_id));
            set_link_id.set(custom_link_id);
        } else if let Some(url_link_id) = get_link_id_from_url() {
            // Found link ID in URL - we're joining an existing connection
            console_log(&format!("Modal using URL link ID: {}", url_link_id));
            set_link_id.set(url_link_id.clone());
            
            // Suggest a default name
            let prefix_len = std::cmp::min(6, url_link_id.len());
            on_name_change.run(String::new());
        } else if !is_view_mode {
            // No link ID in URL - we're creating a new connection
            console_log("Modal requesting new link ID from server");
            // Request a new link ID from the server right away
            request_new_link_id();
        }
    };
    
    // Initialize on component creation
    initialize_link_id(); 

    let (api_error_ref, _) = signal(String::new());
    Effect::new(move |_| {
        // Check if there's a parent component API error about connection being full
        if let Some(window) = web_sys::window() {
            if let Some(error_element) = window.document()
                .and_then(|doc| doc.query_selector(".bg-red-900").ok())
                .flatten() 
            {
                let error_text = error_element.text_content().unwrap_or_default();
                if (error_text.contains("already full") || error_text.contains("already connected")) 
                && !loading_link.get() && link_id.get().is_empty() {
                    // Found an API error about connection being full, generate a new link
                    console_log("Auto-generating new link due to API error: Connection full");
                    request_new_link_id();
                }
            }
        }
    });
    
    // Function to generate the full connection link
    let get_connection_link = move || {
        if loading_link.get() {
            return "Generating link...".to_string();
        }
        
        if !link_error.get().is_empty() {
            return format!("Error: {}", link_error.get());
        }
        
        if link_id.get().is_empty() {
            return "Waiting for link...".to_string();
        }
        
        let window = window().expect("should have window");
        let location = window.location();
        let origin = location.origin().unwrap_or_else(|_| "http://64.181.233.1".to_string());
        let pathname = location.pathname().unwrap_or_else(|_| "/".to_string());
        
        format!("{}{}?link={}", origin, pathname, link_id.get())
    };
    
    view! {
        <div class="fixed inset-0 bg-black bg-opacity-70 flex items-center justify-center z-50">
            <div class="bg-gray-800 p-6 rounded-lg shadow-xl max-w-md w-full mx-4 text-gray-100 border border-gray-700">
                <h3 class="text-xl font-bold mb-4 text-gray-100">
                    {move || {
                        if is_view_mode {
                            "View connection"
                        } else if get_link_id_from_url().is_some() {
                            "Join a connection!"
                        } else {
                            "Make a connection!"
                        }
                    }}
                </h3>
                <div class="flex flex-col gap-4">
                    <div>
                        {move || {
                            if !link_error.get().is_empty() // && (
                                //link_error.get().contains("create a new connection") || 
                                //link_error.get().contains("already full") || 
                                //link_error.get().contains("already connected")
                            //) 
                            {
                                view! {
                                    <div class="mt-2 p-3 bg-yellow-800 text-yellow-100 rounded">
                                        {if link_error.get().contains("already full") {
                                            format!("It looks like that link has already been used! A new connection will be created for {} instead.", connection_name.get())
                                        } else if link_error.get().contains("already connected") {
                                            format!("You're already connected! A new connection will be created for {} instead.", connection_name.get())
                                        } else {
                                            format!("It looks like that link has already been used! You can send this new connection link to {} instead.", connection_name.get())
                                        }}
                                    </div>
                                }.into_any()
                            } else {
                                view! { <></> }.into_any()
                            }
                        }}
                        <label class="block text-sm font-medium mb-1 text-gray-200">
                             "Connect to:"
                        </label>
                        <input
                            type="text"
                            class="w-full px-4 py-2 rounded bg-gray-900 border border-gray-700 text-gray-100 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                            prop:value=connection_name
                            prop:disabled=is_view_mode
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                on_name_change.run(value);
                            }
                        />
                        <div class="mt-1 text-sm text-gray-400">
                            {move || {
                                if get_link_id_from_url().is_some() {
                                    "This is just what you'll see on your list - what do you call the person that sent you the link?"
                                } else {
                                    "This is just what you'll see on your list - they'll set their own account name for themself when they connect"
                                }
                            }}
                        </div>
                        {move || show_name_error.get().then(|| view! {
                            <div class="mt-2 text-red-400 text-sm" data-test-id="connection-name-error">
                                "Please enter a name for your connection."
                            </div>
                        })}
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium mb-1 text-gray-200">
                             "Connection Link"
                        </label>
                        <div class="flex gap-2">
                            <input
                                type="text"
                                class="w-full px-4 py-2 rounded bg-gray-900 border border-gray-700 text-gray-100"
                                readonly=true
                                prop:value=get_connection_link
                            />
                        </div>
                        <div class="mt-1 text-sm text-gray-400">
                            {move || {
                                if get_link_id_from_url().is_some() {
                                    "Using link from URL to join an existing connection"
                                } else if loading_link.get() {
                                    "Generating link..."
                                } else if !link_error.get().is_empty() {
                                    "Error generating link. Will create on submission."
                                } else if !link_id.get().is_empty() {
                                    "Share this link with your friend to connect"
                                } else {
                                    "Waiting for link generation..."
                                }
                            }}
                        </div>
                    </div>

                    <div class="flex justify-end gap-4 mt-4">
                        <button
                            class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-gray-200"
                            on:click=move |_| {
                                on_cancel.run(());
                            }
                        >
                            "Cancel"
                        </button>
                        {move || {
                            if is_view_mode {
                                view! {
                                    <>
                                        <button
                                            class="px-4 py-2 bg-red-600 hover:bg-red-700 rounded text-gray-100"
                                            on:click=move |_| {
                                                if let Some(ref callback) = on_delete {
                                                    callback.run(());
                                                }
                                            }
                                        >
                                            "Delete"
                                        </button>
                                    </>
                                }.into_any()
                            } else {
                                view! {
                                    <button
                                        class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 rounded text-gray-100"
                                        on:click=move |_| on_submit.run(created_connection.get())
                                    >
                                        {if get_link_id_from_url().is_some() { "Join" } else { "Create" }}
                                    </button>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}
