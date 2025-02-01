use leptos::*;

#[component]
fn App() -> impl IntoView {
    view! { 
        <div class="flex min-h-screen items-center justify-center bg-gray-100">
            <h1 class="text-3xl font-bold text-gray-800">"Hello"</h1>
        </div>
    }
}

fn main() {
    mount_to_body(|| view! { <App/> });
}