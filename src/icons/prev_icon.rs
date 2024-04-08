use leptos::{component, view, IntoView};

#[component]
pub fn PrevIcon() -> impl IntoView {
    view! {
        <svg
            class="button-grow"
            width="10"
            height="16"
            viewBox="0 0 10 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M9.24257 2.34317L7.82836 0.928955L0.757324 8.00001L7.82839 15.0711L9.24261 13.6569L3.58574 8L9.24257 2.34317Z"
                fill="var(--textPrimary)"
            ></path>
        </svg>
    }
}
