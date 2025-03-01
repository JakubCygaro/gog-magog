use std::pin::Pin;

use leptos::html::Output;
use leptos::svg::filter;
use leptos::{create_action, document,  Callable, Effect};
use leptos::{component, IntoView, view, prelude::*, expect_context, create_resource, Suspense, Show, NodeRef, create_node_ref};
use web_sys::wasm_bindgen::{self, JsCast};
use web_sys::{js_sys, HtmlBodyElement};
use crate::errors::{self, CreatePostError, WebworksError};
use crate::webworks::{PostsFilter, WebworksResult};
use leptos::logging::{log, warn, error, debug_warn};
use super::data::*;
use super::webworks;

macro_rules! js_closure {
    ($body:expr) => {
        leptos::wasm_bindgen::closure::Closure::wrap(Box::new($body) as Box<dyn FnMut(_)>)
    };
}

pub const LOAD_MORE_AMOUNT: i32 = 10;
pub const LOAD_INITIAL:     i32 = 5;
const INFINITE_LOAD_THRESHHOLD: i32 = 100;

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
    T: Sized + Clone + 'static,
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

        if is_reach_bottom && !load_posts_cooldown_pending.get_untracked() && get_toload.get_untracked().is_none(){
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
                    Err(_) => Some(view!{<p style="text-align:center;">"Failed to load posts!"</p>}.into_view())
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
            {
                move|| {
                    get_posts.get().into_iter().map(|p|{
                        display(p)
                    }).collect::<Vec<_>>()
                }
            }
            {move||{load_posts_pending.get().then(||view!{<p>"loading posts"</p>})}}
        </div>
    }
}

#[component]
pub fn PostsFrontPage() -> impl IntoView {
     
    let user_data = create_resource(|| (), |_| async move { webworks::get_user_data().await });
    let (get_refresh, set_refresh) = create_signal::<Option<()>>(None);
    let dis = move |post: PostData| {
        view! {
            <DisplayPost data=post/>
        }
    };
    view!{
        <div>
            <Suspense
                fallback=move|| view!{ <p>"Cannot create a post while not logged in"</p>}
            >
            {move || {
                user_data.get()
                    .map(|user_data| view! { 
                        <PostForm 
                            user_data=user_data
                            on_posted=move|_|{
                                set_refresh.set(Some(()));
                                //set_toload.set(get_posts.with_untracked(|v|{
                                //    Some((v.len() + 1) as i32)
                                //}));
                                //load_posts.dispatch(get_toload.get_untracked());
                                //load_posts.dispatch(Some(get_toload.get_untracked().unwrap_or(LOAD_INITIAL) + 1));
                            }
                        /> 
                    })
            }}
            </Suspense>

            <h1 style="text-align:center;"> "Newest posts:"</h1><br/>

            //<Posts 
            //    refresh=Some(get_refresh)
            //    post_filter=None/>
            <InfiniteLoad 
                display=dis
                loader=front_posts_loader 
                extra_data={}
                />
        </div>
    }
}
async fn front_posts_loader(_: (), v: i32) -> Result<Vec<PostData>, WebworksError> {
    webworks::load_posts(v, None).await
}

#[component]
pub fn UserPosts() -> impl IntoView {
    let user_data_state = expect_context::<RwSignal<Option<UserData>>>();
    let Some(user_data) = user_data_state.get() else {
        return view! {}.into_view()
    };
    let filter = PostsFilter {
        username: Some(user_data.login),
        limit: None
    };
    let dis = move |post: PostData| {
        view! {
            <DisplayPost data=post/>
        }
    };
    view!{
        <h1 style="text-align:center;"> "Your posts:"</h1><br/>
        <InfiniteLoad 
            display=dis
            loader=user_posts_loader 
            extra_data={filter}
            />
    }.into_view()
}

async fn user_posts_loader(filter: PostsFilter, v: i32) -> Result<Vec<PostData>, WebworksError> {
    webworks::load_posts(v, Some(&filter)).await
}

#[component]
fn PostForm(user_data: Option<UserData>, #[prop(into)] on_posted: leptos::Callback<()>) -> impl IntoView {
    let (get, set) = create_signal(user_data);

    let post_action = create_action(|post_data: &PostCreationData|{
        let post_data = post_data.to_owned();
        async move {
            webworks::create_post(post_data).await
        }
    });
    let post_action_pending = post_action.pending();
    let post_action_value = post_action.value();
    let post_action_outcome = move|| {
        post_action_value.with(|out| {
            match out {
                None => None::<leptos::View>.into_view(),
                Some(res) => match res {
                    Ok(_) => {
                        on_posted.call(());
                        view!{
                            <p style="text-align:center;">"Uploaded!"</p>
                        }.into_view()
                    },
                    Err(err) => {
                        leptos::logging::error!("{:?}", err);
                        match err {
                        CreatePostError::NotLoggedIn => view!{
                            <p style="text-align:center;">"error: User not logged in!"</p>
                        }.into_view(),
                        CreatePostError::ValidationError(val_b) => view!{
                            <p style="text-align:center;">"error: Post data validation error!"</p>
                        }.into_view(),
                        CreatePostError::Webworks { source } => view!{
                            <p style="text-align:center;">"error: An unknown error has occured!"</p>
                        }.into_view(),
                    }
                }
                }.into_view()
            }
        })
    };

    let post_content: NodeRef<leptos::html::Textarea> = create_node_ref();
    let on_click_post = move|_| {
        let content = post_content.get().unwrap().value();
        let pcdata = PostCreationData {
            content: content
        };
        post_action.dispatch(pcdata);
    };

    view!{
        <Show 
        when=move||{get.get().is_some()}
        fallback=|| view!{ <p style="text-align:center;">"You have to be logged in to post"</p> }>

            <div class="flex-container">
                <div class="flex-column">
                    <p
                        style="padding:0;margin:0;text-align:center;"
                    >
                    {move||{get.get().unwrap().login}}
                    </p>
                    <img 
                        src=move|| { webworks::get_pfp_url_for_login(&get.get().unwrap().login)}
                        height="100"
                        width="100"
                        style="padding: 10px;"
                    />
                    <button
                        style="padding: 5px; "
                        on:click=on_click_post
                        type="button">
                        "Post"
                    </button>
                    
                </div>
                <textarea type="text" wrap="hard" rows="5"
                            class="post-textbox"
                            prop:value="Post text"
                            maxlength="300"
                            node_ref=post_content
                            />
            </div>
            {move||{post_action_pending.get().then(||view!{<p style="text-align:center;">"Uploading your post"</p>})}}
            {post_action_outcome}
        </Show>

    }
}

#[component]
fn DisplayPost(data: PostData) -> impl IntoView {
    let (get_data, _set_data) = create_signal(data);
    view! {
        <div class="flex-container posts-section">
            <div class="flex-column">
                <a 
                    href=move||{format!("users?name={}", get_data.get().login)}
                    class="user-profile-link"
                >{
                    move||{get_data.get().login}
                }</a>
                <img 
                    src=move|| { webworks::get_pfp_url_for_login(get_data.get().login.as_str())}
                    height="100"
                    width="100"
                    style="padding: 10px;display: block;
                                    margin-left: auto;
                                    margin-right: auto;"
                />
                <p style="padding:0;margin:0;text-align:center;">
                    {
                        move||{
                            get_data.get().posted.format("%Y-%m-%d %H:%M").to_string()
                        }
                    }
                </p>
            </div>
            <textarea type="text" wrap="hard" rows="5"
                    class="post-textbox"
                    prop:value=move||{get_data.get().content}
                    maxlength="300"
                    readonly
            />
        </div>
    }
}

