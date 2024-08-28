use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use leptos::{
    as_child_of_current_owner, component, create_effect, create_isomorphic_effect, create_memo,
    create_node_ref, create_render_effect, create_rw_signal, create_signal, html::Div,
    provide_context, set_timeout, view, IntoView, NodeRef, RwSignal, SignalGet, SignalGetUntracked,
    SignalSet, SignalWith, TextProp, View,
};
use leptos_router::{use_location, use_route, use_router, Outlet, RouteContext};
use web_sys::AnimationEvent;

fn get_class_str(
    class: Option<TextProp>,
    additional_class: Option<&'static str>,
) -> Option<String> {
    if let Some(class) = &class {
        if let Some(additional_class) = &additional_class {
            return Some(format!("{} {}", class.get(), additional_class));
        } else {
            return Some(class.get().to_string());
        }
    }
    None
}

fn set_classes(
    class_setter1: RwSignal<Option<String>>,
    class_setter2: RwSignal<Option<String>>,
    default_class: Option<TextProp>,
    class1: Option<&'static str>,
    class2: Option<&'static str>,
) {
    class_setter1.set(get_class_str(default_class.clone(), class1));
    class_setter2.set(get_class_str(default_class, class2));
}

#[component]
pub fn AnimatedOutletSimultaneous(
    #[prop(optional, into)] class: Option<TextProp>,
    #[prop(optional)] intro: Option<&'static str>,
    #[prop(optional)] outro: Option<&'static str>,
) -> impl IntoView {
    let pathname = use_location().pathname;
    let route = use_route();

    let current_outlet = create_rw_signal(None::<View>);
    let next_outlet = create_rw_signal(None::<View>);

    let current_outlet_class = create_rw_signal(get_class_str(class.clone(), None));
    let next_outlet_class = create_rw_signal(get_class_str(class.clone(), None));

    let class_clone = class.clone();
    let build_outlet = as_child_of_current_owner(
        move |(child, outlet_class, is_current_outlet): (
            RouteContext,
            RwSignal<Option<String>>,
            bool,
        )| {
            provide_context(child.clone());

            let class = class_clone.clone();
            let mut has_animated_once = false;

            let mut on_animation_end = move |ev: AnimationEvent, node_ref: NodeRef<Div>| {
                if has_animated_once {
                    return;
                }

                use wasm_bindgen::JsCast;
                if let Some(target) = ev.target() {
                    let node_ref = node_ref.get();
                    if node_ref.is_none()
                        || target
                            .unchecked_ref::<web_sys::Node>()
                            .is_same_node(Some(&*node_ref.unwrap()))
                    {
                        ev.stop_propagation();
                        set_classes(
                            current_outlet_class,
                            next_outlet_class,
                            class.clone(),
                            None,
                            None,
                        );
                        if is_current_outlet {
                            next_outlet.set(None)
                        } else {
                            current_outlet.set(None)
                        }
                        has_animated_once = true;
                    }
                }
            };

            view! {
                <>
                    {
                        let node_ref = create_node_ref::<Div>();
                        view! {
                            <div
                                on:animationend=move |ev| on_animation_end(ev, node_ref)
                                node_ref=node_ref
                                class=move || outlet_class.get()
                            >
                                {child.outlet().into_view()}
                            </div>
                        }
                    }
                </>
            }
            .into_view()
        },
    );

    create_render_effect(move |_| {
        pathname.track();

        let child = route.child();
        if let Some(child) = child {
            let disposer = if next_outlet.get_untracked().is_none() {
                let (outlet, disposer) = build_outlet((child, next_outlet_class, false));
                next_outlet.set(Some(outlet));

                set_classes(
                    current_outlet_class,
                    next_outlet_class,
                    class.clone(),
                    outro,
                    intro,
                );
                disposer
            } else {
                let (outlet, disposer) = build_outlet((child, current_outlet_class, true));
                current_outlet.set(Some(outlet));
                set_classes(
                    next_outlet_class,
                    current_outlet_class,
                    class.clone(),
                    outro,
                    intro,
                );
                disposer
            };

            return Some(disposer);
        }

        None
    });

    view! { <>{move || { next_outlet.get() }} {move || { current_outlet.get() }}</> }
}
