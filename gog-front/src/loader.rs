use std::pin::Pin;
use std::str::FromStr;

use leptos::html::Output;
use leptos::svg::filter;
use leptos::{create_action, document,  Callable, Effect};
use leptos::{component, IntoView, view, prelude::*, expect_context, create_resource, Suspense, Show, NodeRef, create_node_ref, For};
use web_sys::wasm_bindgen::{self, JsCast};
use web_sys::{js_sys, HtmlBodyElement};
use crate::errors::{self, CreatePostError, WebworksError};
use crate::webworks::{PostsFilter, WebworksResult};
use leptos::logging::{log, warn, error, debug_warn};

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
    #[prop(default = None)]
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
    //how many posts should be loaded (None if none new posts should be downloaded)
    let (get_toload, set_toload) = create_signal(None);

    let load_posts_cooldown = create_action(move|_:&()|{
        use gloo_timers::future::TimeoutFuture;

        async {
            TimeoutFuture::new(5_000).await
        }
    });

    let load_posts_cooldown_pending = load_posts_cooldown.pending();
    
    //this action downloads posts based on the to_load signal, it loads no posts if to_load is set
    //to None
    let load_posts = create_action(move |to_load: &Option<i32>| {
        let to_load = to_load.to_owned();
        let loader = loader.clone();
        let extra_data = extra_data.clone();
        async move {
            if let Some(v) = to_load {
                let s = loader(extra_data.clone(), v).await;
                if let Ok(posts) = s {
                    if posts.is_empty() && get_posts.with_untracked(|p| p.is_empty()) {
                        return Ok(())
                    } else if posts.len() > get_posts.with_untracked(|p| p.len()) {
                        set_posts.set(posts);
                        set_toload.set(None);
                    } else {
                        load_posts_cooldown.dispatch(());
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

        if is_reach_bottom && !load_posts_cooldown_pending.get() && get_toload.get().is_none(){
                set_toload.set(Some(get_posts.with_untracked(|v| v.len() + load_more as usize) as i32));
                load_posts.dispatch(get_toload.get_untracked());
        }
    });

    window.set_onscroll(Some(onscroll.as_ref().unchecked_ref()));
    onscroll.forget();

    leptos::create_effect(move |_|{
        load_posts.dispatch(Some(load_initial));
    });

    let load_posts_value = load_posts.value();
    let load_posts_pending = load_posts.pending();
    let display_posts = move|| {
        load_posts_value.with(|v|{
            match v {
                None => None,
                Some(res) => match res {
                    Ok(()) => None,
                    Err(_) => Some(view!{<p style="text-align:center;">"Failed to load data"</p>}.into_view())
                }
            }
        }) 
    };
    if let Some(refresh) = refresh {
        let _refresh_fn = Effect::new(move |_| {
            if refresh.get().is_none() {
                return;
            };
            log!("refresh requested!");
            set_toload.set(get_posts.with_untracked(|v|{
                Some((v.len()) as i32)
            }));
            //load_posts.dispatch(get_toload.get_untracked());
            load_posts.dispatch(Some(get_toload.get_untracked().unwrap_or(load_initial) + 1));
        });
    }
    view!{
        <div
            on:load=move|_|{
                leptos::logging::log!("div_on_load");
                load_posts.dispatch(Some(10));
            }>

            {display_posts}
            //{
            //    move|| {
            //        get_posts.get().into_iter().map(|p|{
            //            display(p)
            //        }).collect::<Vec<_>>()
            //    }
            //}
            <For
                each=move || get_posts.get()
                key=|state| state.key()
                children=move|item| {
                    view!{
                        {display(item)}
                    }.into_view()
                }
                />
            {move||{load_posts_pending.get().then(||view!{<p>"loading posts"</p>})}}
        </div>
    }
}
