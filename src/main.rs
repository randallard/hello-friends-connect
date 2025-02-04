use hello_leptos::App;
use leptos::*;
use leptos::prelude::*;

fn main() {
    mount_to_body(|| view! { <App/> });
}