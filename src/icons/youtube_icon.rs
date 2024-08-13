use leptos::{component, view, IntoView};

#[component]
pub fn YoutubeIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 21 16" fill="none" xmlns="http://www.w3.org/2000/svg">
            <title>Youtube</title>
            <g>
                <rect x="9" y="5" width="5" height="6" fill="white" />
                <path
                    d="M20.5933 2.80322C20.4794 2.3454 20.2568 1.92789 19.9477 1.59223C19.6386 1.25658 19.2537 1.01448 18.8313 0.890051C17.2653 0.424217 11.0003 0.416634 11.0003 0.416634C11.0003 0.416634 4.73633 0.409051 3.16933 0.854301C2.74725 0.984459 2.36315 1.22998 2.0539 1.56728C1.74464 1.90458 1.52062 2.32235 1.40333 2.78047C0.99033 4.47697 0.98633 7.99564 0.98633 7.99564C0.98633 7.99564 0.98233 11.5316 1.39233 13.2108C1.62233 14.1392 2.29733 14.8726 3.15533 15.1229C4.73733 15.5887 10.9853 15.5963 10.9853 15.5963C10.9853 15.5963 17.2503 15.6039 18.8163 15.1597C19.2388 15.0355 19.6241 14.794 19.934 14.459C20.2439 14.124 20.4677 13.7072 20.5833 13.2498C20.9973 11.5544 21.0003 8.0368 21.0003 8.0368C21.0003 8.0368 21.0203 4.49972 20.5933 2.80322V2.80322ZM8.99633 11.2554L9.00133 4.75539L14.2083 8.0108L8.99633 11.2554V11.2554Z"
                    fill="#E62017"
                />
            </g>

            <defs>
                <filter
                    id="filter0_d"
                    x="0"
                    y="0"
                    width="30"
                    height="25"
                    filterUnits="userSpaceOnUse"
                    color-interpolation-filters="sRGB"
                >
                    <feFlood flood-opacity="0" result="BackgroundImageFix" />
                    <feColorMatrix
                        in="SourceAlpha"
                        type="matrix"
                        values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0"
                    />
                    <feOffset dy="4" />
                    <feGaussianBlur stdDeviation="2" />
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
