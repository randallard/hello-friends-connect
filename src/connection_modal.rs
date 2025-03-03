use wasm_bindgen_test::*;
use leptos::*;
use leptos::prelude::*;
use web_sys::console::log;
use crate::connection_utils::get_link_id_from_url; 

#[component]
pub fn ConnectionModal(
    #[prop(into)] connection_name: Signal<String>,
    #[prop(into)] show_name_error: Signal<bool>,
    #[prop(into)] on_name_change: Callback<String>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_submit: Callback<()>,
) -> impl IntoView {

    let url_link_id = get_link_id_from_url();
    if let Some(ref link_id) = url_link_id {
        console_log!("ConnectionModal found URL link ID: {}", link_id);
    } else {
        console_log!("ConnectionModal did not find a URL link ID");
    }
    
    // Check for URL link ID and set default name if found
    if let Some(link_id) = url_link_id {
        if !link_id.is_empty() {
            let prefix_len = std::cmp::min(6, link_id.len());
            let default_name = format!("Connection {}", &link_id[..prefix_len]);
            console_log!("Setting default name: {}", default_name);
            on_name_change.run(default_name);
        }
    }

    view! {
        <div class="fixed inset-0 bg-black bg-opacity-70 flex items-center justify-center z-50">
            <div class="bg-gray-800 p-6 rounded-lg shadow-xl max-w-md w-full mx-4 text-gray-100 border border-gray-700">
                <h3 class="text-xl font-bold mb-4 text-gray-100">"Make a connection!"</h3>
                <div class="flex flex-col gap-4">
                    <div>
                        <label class="block text-sm font-medium mb-1 text-gray-200">
                             "Connect to:"
                        </label>
                        <input
                            type="text"
                            class="w-full px-4 py-2 rounded bg-gray-900 border border-gray-700 text-gray-100 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                            prop:value=connection_name
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                on_name_change.run(value);
                            }
                        />
                        <div class="mt-1 text-sm text-gray-400">
                            "This is just what you'll see on your list - they'll set their own account name for themself when they connect"
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
                        <input
                            type="text"
                            class="w-full px-4 py-2 rounded bg-gray-900 border border-gray-700 text-gray-100"
                            readonly=true
                            value="connection link will appear here"
                        />

                        <div class="flex justify-end gap-4 mt-4">
                            <button
                                class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-gray-200"
                                on:click=move |_| {
                                    on_cancel.run(());
                                }
                            >
                                "Cancel"
                            </button>
                            <button
                                class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 rounded text-gray-100"
                                on:click=move |_| {
                                    on_submit.run(());
                                }
                            >
                                "OK"
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}