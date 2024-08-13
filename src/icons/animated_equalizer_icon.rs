use leptos::{component, view, IntoView};

#[component]
pub fn AnimatedEqualizerIcon(#[prop()] playing: bool) -> impl IntoView {
    view! {
        <svg
            id="eF20KXoiB5d1"
            xmlns="http://www.w3.org/2000/svg"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            viewBox="0 0 12 14"
            shape-rendering="geometricPrecision"
            text-rendering="geometricPrecision"
            class="animated-playing"
        >
            <g
                id="eF20KXoiB5d2_ts"
                class=("animation-play", playing)
                class=("animation-pause", !playing)
                transform="translate(1.989899,14) scale(1,1)"
            >
                <path
                    id="eF20KXoiB5d2"
                    d="M3.337220,0.381592L0.642578,0.381592L0.642578,13.991600L3.337220,13.991600L3.337220,0.381592Z"
                    transform="translate(-1.989899,-13.991600)"
                    fill="#ffffff"
                    stroke="none"
                    stroke-width="1"
                    stroke-miterlimit="1"
                />
            </g>
            <g
                id="eF20KXoiB5d3_ts"
                class=("animation-play", playing)
                class=("animation-pause", !playing)
                transform="translate(6.030915,14.000671) scale(1,1)"
            >
                <path
                    id="eF20KXoiB5d3"
                    d="M7.378240,5.199710L4.683590,5.199710L4.683590,13.991700L7.378240,13.991700L7.378240,5.199710Z"
                    transform="translate(-6.030915,-13.991600)"
                    fill="#ffffff"
                    stroke="none"
                    stroke-width="1"
                    stroke-miterlimit="1"
                />
            </g>
            <g
                id="eF20KXoiB5d4_ts"
                class=("animation-play", playing)
                class=("animation-pause", !playing)
                transform="translate(9.851647,14.039325) scale(1,1)"
            >
                <path
                    id="eF20KXoiB5d4"
                    d="M11.175100,8.803340L8.480470,8.803340L8.480470,13.991600L11.175100,13.991600L11.175100,8.803340Z"
                    transform="translate(-9.827785,-13.991600)"
                    fill="#ffffff"
                    stroke="none"
                    stroke-width="1"
                    stroke-miterlimit="1"
                />
            </g>
        </svg>
    }
}
