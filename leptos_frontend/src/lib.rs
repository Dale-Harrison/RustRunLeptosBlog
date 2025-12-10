use leptos::*;

#[component]
pub fn App() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <div class="p-8 text-center font-sans">
            <h1 class="text-4xl font-bold text-blue-600 mb-4">"Hello from Leptos (Rust inside!)"</h1>
            <p class="mb-4 text-xl">"This entire component is compiled to WebAssembly."</p>
            <button
                class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition"
                on:click=move |_| set_count.update(|n| *n += 1)
            >
                "Click me: " {count}
            </button>
        </div>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
