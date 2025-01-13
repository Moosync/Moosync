use leptos::{component, prelude::*, view, IntoView};

#[tracing::instrument(level = "trace", skip(title))]
#[component]
pub fn PlainPlayIcon(#[prop()] title: String) -> impl IntoView {
    view! {
        <svg
            class="button-grow"
            style="cursor: pointer;"
            width="22"
            height="22"
            viewBox="0 0 20 20"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <title>{title}</title>
            <path
                d="M1.2236 0.552783C1.06861 0.475289 0.884538 0.483573 0.737132 0.574676C0.589725 0.66578 0.5 0.826712 0.5 1V19C0.5 19.1733 0.589727 19.3342 0.737137 19.4253C0.884547 19.5164 1.06862 19.5247 1.22361 19.4472L19.2236 10.4469C19.393 10.3622 19.5 10.1891 19.5 9.99969C19.5 9.8103 19.393 9.63717 19.2236 9.55247L1.2236 0.552783Z"
                fill="var(--accent)"
                stroke="var(--accent)"
                stroke-linecap="round"
                stroke-linejoin="round"
            ></path>
        </svg>
    }
}
