use leptos::logging::*;
use leptos::{create_action, Effect};
use leptos::{component, IntoView, view, prelude::*, For};
use web_sys::wasm_bindgen::{JsCast};
use leptos::logging::{log, error};

pub const LOAD_MORE_AMOUNT: i32 = 10;
pub const LOAD_INITIAL:     i32 = 5;
const INFINITE_LOAD_THRESHHOLD: i32 = 100;
pub trait HasKey {
    fn key(&self) -> uuid::Uuid;
}
#[component]
pub fn InfiniteLoad<T, S, L, D, IV, E, F>(
    #[prop(default = LOAD_MORE_AMOUNT)]
    load_more: i32,
    #[prop(default = LOAD_INITIAL)]
    load_initial: i32,
    #[prop(optional)]
    refresh: Option<ReadSignal<Option<()>>>,
    extra_data: S,
    loader: L,
    display: D)
    -> impl IntoView
where
    T: Sized + Clone + HasKey + 'static,
    S: Sized + Clone + 'static,
    E: std::error::Error + 'static,
    F: std::future::Future<Output = Result<Vec<T>, E>>,
    L: Fn(S, i32) -> F + 'static + Clone,
    IV: IntoView,
    D: Fn(T) -> IV + 'static,
{
    //loaded posts cache
    let (get_posts, set_posts) =
        create_signal::<Vec<T>>(vec![]);
    //how many posts should be loaded (None if no new posts should be downloaded)
    let (get_toload, set_toload) = create_signal(None);

    let load_items_cooldown = create_action(move|_:&()|{
        use gloo_timers::future::TimeoutFuture;

        async {
            TimeoutFuture::new(5_000).await
        }
    });

    let load_items_cooldown_pending = load_items_cooldown.pending();
    //this action downloads posts based on the to_load signal, it loads no posts if to_load is set
    //to None
    let load_items = create_action(move |to_load: &Option<i32>| {
        let to_load = to_load.to_owned();
        let loader = loader.clone();
        let extra_data = extra_data.clone();
        async move {
            if let Some(v) = to_load {
                debug_warn!("to_load: {}", v);
                let s = loader(extra_data.clone(), v).await;
                if let Ok(posts) = s {
                    if posts.is_empty() && get_posts.with_untracked(|p| p.is_empty()) {
                        return Ok(())
                    } else if posts.len() > get_posts.with_untracked(|p| p.len()) {
                        leptos::logging::debug_warn!("new posts loaded");
                        set_posts.set(posts);
                        set_toload.set(None);
                    } else {
                        leptos::logging::debug_warn!("cooldown set");
                        load_items_cooldown.dispatch(());
                        set_toload.set(Some(get_toload.with_untracked(|v| v.unwrap_or(load_more) - load_more)));
                    }
                    return Ok(())
                } else {
                    let err = s.err().unwrap();
                    error!("{}", err);
                    return Err(err);
                }
            }
            Ok(())
        }
    });

    let window = leptos::window();

    let onscroll = js_closure!(move|e: web_sys::Event| {
        let window = leptos::window();
        let body = leptos::document().body().unwrap();
        let scrolled_to = window.scroll_y().unwrap() + window.inner_height().unwrap().as_f64().unwrap();
        let is_reach_bottom = body.scroll_height() - INFINITE_LOAD_THRESHHOLD <= scrolled_to as i32;

        if is_reach_bottom && !load_items_cooldown_pending.get_untracked() && get_toload.get_untracked().is_none(){
                set_toload.set(Some(get_posts.with_untracked(|v| v.len() + load_more as usize) as i32));
                load_items.dispatch(get_toload.get_untracked());
        }
    });

    window.set_onscroll(Some(onscroll.as_ref().unchecked_ref()));
    onscroll.forget();

    leptos::create_effect(move |_|{
        load_items.dispatch(Some(load_initial));
    });

    let load_items_value = load_items.value();
    let load_items_pending = load_items.pending();
    let display_posts = move|| {
        load_items_value.with(|v|{
            match v {
                None => None,
                Some(res) => match res {
                    Ok(()) => None,
                    Err(_) => Some(view!{<p style="text-align:center;">"Failed to load data"</p>}.into_view())
                }
            }
        })
    };
    view!{
        <div
            on:load=move|_|{
                leptos::logging::log!("div_on_load");
                load_items.dispatch(Some(10));
            }>

            {display_posts}
            <For
                each=move || get_posts.get()
                key=|state| state.key()
                children=move|item| {
                    view!{
                        {display(item)}
                    }.into_view()
                }
                />
            {move||{load_items_pending.get().then(||view!{<p>"loading..."</p>})}}
            {
                if let Some(refresh) = refresh {
                    view!{
                        {
                            let _refresh_fn = move || {
                                if refresh.get().is_none() {
                                    return;
                                };
                                set_toload.set(get_posts.with_untracked(|v|{
                                    Some((v.len()) as i32)
                                }));
                                load_items.dispatch(Some(get_toload.get_untracked().unwrap_or(load_initial) + 1));
                            };
                            Some(_refresh_fn)
                        }
                    }
                } else {
                    None
                }
            }
        </div>
    }
}
