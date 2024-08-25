use leptos::{component, view, IntoView};

#[component]
pub fn NewPlaylistIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 68 68" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path
                d="M30.9275 9.93652H12.1407C9.92622 9.93652 7.80245 10.8162 6.23658 12.3821C4.67071 13.948 3.79102 16.0717 3.79102 18.2862V55.8598C3.79102 58.0742 4.67071 60.198 6.23658 61.7639C7.80245 63.3298 9.92622 64.2094 12.1407 64.2094H53.8891C56.1036 64.2094 58.2273 63.3298 59.7932 61.7639C61.3591 60.198 62.2388 58.0742 62.2388 55.8598V37.073"
                stroke="var(--textPrimary)"
                stroke-width="7"
                stroke-linecap="round"
                stroke-linejoin="round"
            ></path>
            <path
                d="M54 15L57.3773 18.4555M61.7484 5.79316C62.8613 6.94398 63.4776 8.486 63.4643 10.0869C63.451 11.6878 62.8093 13.2194 61.6774 14.3516L32.5245 43.5045L20 47.6793L24.1748 35.1548L53.3444 5.73889C54.3699 4.70552 55.7422 4.08914 57.1958 4.00894C58.6494 3.92874 60.0811 4.39042 61.214 5.3047L61.7484 5.79316Z"
                stroke="var(--textPrimary)"
                stroke-width="4"
                stroke-linecap="round"
                stroke-linejoin="round"
            ></path>
        </svg>
    }
}
