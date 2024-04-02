use leptos::{component, view, IntoView};

#[component]
pub fn ExpandIcon() -> impl IntoView {
    view! {
        <svg class="button-grow" viewBox="0 0 22 13" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path
                d="M1.00049 12L11.0005 2L21.0005 12"
                stroke="var(--accent)"
                stroke-width="2"
            ></path>
        </svg>
    }
}
