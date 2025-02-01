// src/connect_component.rs
use leptos::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[component]
pub fn FriendsConnect(
    #[prop(default = "http://localhost:8080")] api_base: &'static str,
) -> impl IntoView {
    view! {
        <div class="max-w-md mx-auto p-4">
            <h2 class="text-xl font-bold mb-4">"Connect with Friends"</h2>
        </div>
    }
}

#[wasm_bindgen_test]
fn test_friends_connect_renders() {
    mount_to_body(|| view! { <FriendsConnect /> });
    
    let heading = document()
        .query_selector("h2")
        .unwrap()
        .unwrap();
        
    assert_eq!(heading.text_content().unwrap(), "Connect with Friends");
}