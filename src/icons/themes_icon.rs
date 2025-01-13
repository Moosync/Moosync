use leptos::{component, prelude::*, view, IntoView};

#[tracing::instrument(level = "trace", skip(active))]
#[component]
pub fn ThemesIcon(#[prop()] active: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg
            width="19"
            height="19"
            viewBox="0 0 19 19"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M9.50004 17.4166C13.8724 17.4166 17.4167 13.8724 17.4167 9.49998C17.4167 5.1276 13.8724 1.58331 9.50004 1.58331C5.12767 1.58331 1.58337 5.1276 1.58337 9.49998C1.58337 13.8724 5.12767 17.4166 9.50004 17.4166ZM9.50004 15.8333V3.16665C11.1797 3.16665 12.7907 3.83391 13.9784 5.02164C15.1661 6.20937 15.8334 7.82027 15.8334 9.49998C15.8334 11.1797 15.1661 12.7906 13.9784 13.9783C12.7907 15.1661 11.1797 15.8333 9.50004 15.8333V15.8333Z"
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
            />
        </svg>
    }
}
