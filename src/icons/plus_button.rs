use leptos::{component, prelude::*, view, IntoView};

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn PlusIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 63 63" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path
                d="M31.5002 59.1667V31.5M31.5002 31.5V3.83337M31.5002 31.5H59.1668M31.5002 31.5H3.8335"
                stroke="var(--textPrimary)"
                stroke-width="6.91667"
                stroke-linecap="round"
            ></path>
        </svg>
    }
}
