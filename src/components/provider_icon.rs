use leptos::{component, create_rw_signal, view, IntoView, SignalGet};

#[component]
pub fn ProviderIcon(#[prop()] extension: Option<String>) -> impl IntoView {
    let provider_icon = create_rw_signal(None::<&str>);
    view! {
        <div class="d-flex provider-icon">
            {move || {
                if provider_icon.get().is_some() {
                    view! { <img src=""/> }.into_view()
                } else {
                    view! {}.into_view()
                }
            }}

        </div>
    }
}
