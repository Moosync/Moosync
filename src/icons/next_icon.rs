use leptos::{component, view, IntoView};

#[component]
pub fn NextIcon() -> impl IntoView {
    view! {
        <svg
            class="button-grow"
            width="9"
            height="16"
            viewBox="0 0 9 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M0.0859375 2.34317L1.50015 0.928955L8.57122 8.00002L1.50015 15.0711L0.0859375 13.6569L5.74279 8.00002L0.0859375 2.34317Z"
                fill="var(--textPrimary)"
            ></path>
        </svg>
    }
}
