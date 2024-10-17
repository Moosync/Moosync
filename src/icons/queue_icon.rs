use leptos::{component, view, IntoView, ReadSignal, SignalGet};

#[tracing::instrument(level = "trace", skip(active))]
#[component]
pub fn QueueIcon(#[prop()] active: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg
            width="18"
            height="18"
            viewBox="0 0 24 21"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >

            <title>Queue</title>
            <path
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
                d="M6 0.666504H24V2.6665H6V0.666504ZM6 6.6665H24V8.6665H6V6.6665ZM11 12.6665H24V14.6665H11V12.6665ZM6 18.6665H24V20.6665H6V18.6665ZM0 8.6665L7 13.6665L0 18.6665V8.6665Z"
            ></path>
        </svg>
    }
}
