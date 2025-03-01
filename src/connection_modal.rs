use leptos::*;
use leptos::prelude::*;

#[component]
pub fn ConnectionModal(
    #[prop(into)] connection_name: Signal<String>,
    #[prop(into)] show_name_error: Signal<bool>,
    #[prop(into)] on_name_change: Callback<String>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_submit: Callback<()>,
) -> impl IntoView {
    view! {
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
                            on:input=move |ev| on_name_change(event_target_value(&ev))
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
                            on:click=move |_| on_cancel()
                        >
                            "Cancel"
                        </button>
                        <button
                            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded"
                            on:click=move |_| on_submit()
                        >
                            "OK"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}