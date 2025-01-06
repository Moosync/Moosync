use leptos::{component, view, IntoView, ReadSignal, SignalGet};

#[tracing::instrument(level = "trace", skip(active))]
#[component]
pub fn AllSongsIcon(#[prop()] active: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg
            width="18"
            height="18"
            viewBox="0 0 18 18"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <title>Songs</title>
            <path
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M1.00015 16.5L2.00015 17.5C2.51233 18.0122 3.48782 18.0122 4 17.5L11.3596 10.8608C13.119 11.3441 15.0811 10.8945 16.4635 9.51203C18.5122 7.4633 18.5122 4.14166 16.4635 2.09294C14.4148 0.0442154 11.0931 0.0442154 9.04442 2.09294C7.66335 3.47401 7.21328 5.43352 7.69419 7.1916C7.79613 7.62773 8.2 8.7 9 9.5C9 9.5 7.11375 9.70679 6 8.98081C3.79014 11.3934 1.00005 14.5 1.00005 14.5C0.487869 15.0122 0.487974 15.9878 1.00015 16.5Z"
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
            ></path>
        </svg>
    }
}
