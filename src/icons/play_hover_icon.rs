use leptos::{component, view, IntoView};

#[component]
pub fn PlayHoverIcon() -> impl IntoView {
    view! {
        <svg
            width="23"
            height="26"
            viewBox="0 0 23 26"
            fill="none"
            style="cursor: pointer;"
            class="align-self-center"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M21.789 10.9007L3.71707 0.333515C2.24872 -0.524653 0 0.308125 0 2.4307V23.5599C0 25.4641 2.08957 26.6117 3.71707 25.6571L21.789 15.095C23.4011 14.1556 23.4062 11.8401 21.789 10.9007V10.9007Z"
                fill="white"
            ></path>
        </svg>
    }
}
