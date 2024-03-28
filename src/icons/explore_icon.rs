use leptos::{component, view, IntoView, ReadSignal, SignalGet};

#[component]
pub fn ExploreIcon(#[prop()] active: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg
            width="18"
            height="18"
            viewBox="0 0 18 18"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M11.75 11.7501C9.4028 14.0973 5.5972 14.0973 3.24998 11.7501C0.902752 9.40285 0.902752 5.59725 3.24998 3.25002C5.5972 0.902801 9.4028 0.902801 11.75 3.25002C14.0972 5.59725 14.0972 9.40285 11.75 11.7501ZM11.75 11.7501L16.5 16.5"
                stroke=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
                stroke-width="2"
                stroke-linejoin="round"
            ></path>
        </svg>
    }
}
