use leptos::{component, prelude::*, view, IntoView};

#[tracing::instrument(level = "trace", skip(accent))]
#[component]
pub fn SearchIcon(#[prop()] accent: bool) -> impl IntoView {
    view! {
        <svg
            style="cursor: pointer;"
            width="19"
            height="19"
            viewBox="0 0 19 19"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M12.3431 12.2426C14.6863 9.8995 14.6863 6.1005 12.3431 3.75736C10 1.41421 6.20101 1.41421 3.85786 3.75736C1.51472 6.1005 1.51472 9.8995 3.85786 12.2426C6.20101 14.5858 10 14.5858 12.3431 12.2426ZM13.7574 2.34315C16.6425 5.22833 16.8633 9.76899 14.4195 12.9075C14.4348 12.921 14.4498 12.9351 14.4645 12.9497L18.7071 17.1924C19.0976 17.5829 19.0976 18.2161 18.7071 18.6066C18.3166 18.9971 17.6834 18.9971 17.2929 18.6066L13.0503 14.364C13.0356 14.3493 13.0215 14.3343 13.008 14.319C9.8695 16.7628 5.32883 16.542 2.44365 13.6569C-0.680542 10.5327 -0.680542 5.46734 2.44365 2.34315C5.56785 -0.781049 10.6332 -0.781049 13.7574 2.34315Z"
                fill=if accent { "var(--accent)" } else { "var(--textPrimary)" }
            ></path>
        </svg>
    }
}
