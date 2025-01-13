use leptos::{component, prelude::*, view, IntoView};

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn NewThemeIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 401 230" fill="none" xmlns="http://www.w3.org/2000/svg">
            <g filter="url(#filter0_d)">
                <rect
                    x="4.00002"
                    y="1.38674"
                    width="393.713"
                    height="222.613"
                    rx="7.35912"
                    fill="var(--secondary)"
                    stroke="#6B6B6B"
                    stroke-width="1.22652"
                />
                <path
                    d="M201 32C203.685 32 206.261 33.0667 208.159 34.9655C210.058 36.8643 211.125 39.4397 211.125 42.125V102.875H271.875C274.56 102.875 277.136 103.942 279.034 105.841C280.933 107.739 282 110.315 282 113C282 115.685 280.933 118.261 279.034 120.159C277.136 122.058 274.56 123.125 271.875 123.125H211.125V183.875C211.125 186.56 210.058 189.136 208.159 191.034C206.261 192.933 203.685 194 201 194C198.315 194 195.739 192.933 193.841 191.034C191.942 189.136 190.875 186.56 190.875 183.875V123.125H130.125C127.44 123.125 124.864 122.058 122.966 120.159C121.067 118.261 120 115.685 120 113C120 110.315 121.067 107.739 122.966 105.841C124.864 103.942 127.44 102.875 130.125 102.875H190.875V42.125C190.875 39.4397 191.942 36.8643 193.841 34.9655C195.739 33.0667 198.315 32 201 32V32Z"
                    fill="var(--textSecondary)"
                />
            </g>
            <defs>
                <filter
                    id="filter0_d"
                    x="0.93368"
                    y="0.773438"
                    width="399.845"
                    height="228.746"
                    filterUnits="userSpaceOnUse"
                    color-interpolation-filters="sRGB"
                >
                    <feFlood flood-opacity="0" result="BackgroundImageFix" />
                    <feColorMatrix
                        in="SourceAlpha"
                        type="matrix"
                        values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0"
                        result="hardAlpha"
                    />
                    <feOffset dy="2.45304" />
                    <feGaussianBlur stdDeviation="1.22652" />
                    <feComposite in2="hardAlpha" operator="out" />
                    <feColorMatrix
                        type="matrix"
                        values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0.25 0"
                    />
                    <feBlend mode="normal" in2="BackgroundImageFix" result="effect1_dropShadow" />
                    <feBlend
                        mode="normal"
                        in="SourceGraphic"
                        in2="effect1_dropShadow"
                        result="shape"
                    />
                </filter>
            </defs>
        </svg>
    }
}
