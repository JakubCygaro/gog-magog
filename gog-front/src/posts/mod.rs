use super::loader::*;
use chrono::TimeZone;
use leptos_router::{use_query, NavigateOptions};
use std::str::FromStr;

use leptos::{create_action,  Callable};
use leptos::{component, IntoView, Suspense, view, prelude::*, expect_context, create_resource, Show, NodeRef, create_node_ref};
use crate::errors::{CreatePostError, WebworksError};
use crate::webworks::PostsFilter;
use super::data::*;
use super::webworks;


impl HasKey for PostData {
    fn key(&self) -> uuid::Uuid {
        uuid::Uuid::from_str(&self.post_id).expect("failed to parse post_id as uuid")
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
fn DisplayPost(data: PostData, #[prop(default = true)] comment_button: bool) -> impl IntoView {
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
                            let posted = get_data.get().posted;
                            let posted = chrono::Local{}.from_utc_datetime(&posted.naive_local());
                            posted.format("%Y-%m-%d %H:%M").to_string()
                        }
                    }
                </p>
                {move||{
                           comment_button.then(move||view!{
                               <button
                                   on:click=move|ev|{
                                       ev.prevent_default();
                                       let nav = leptos_router::use_navigate();
                                       let post = format!("/post?id={}", get_data.get().post_id);
                                       nav(&post, NavigateOptions::default());
                                   }>
                                   "Comments"
                               </button>
                           })
                       }
                }
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

#[component]
pub fn Post() -> impl IntoView {

    use crate::util::AwaitWithError;
    let query = use_query::<PostQuery>();

    if query.with_untracked(|q| q.is_err()){
        return view!{<super::NotFound/>};
    }
    let err_handler=move|err: &WebworksError| {
        leptos::logging::error!("{}",err);
        view!{
            <p>"An error has occured"</p>
        }.into_view()
    };
    let comment_display = |cdata: CommentData| {
        view!{
            <crate::comments::DisplayComment data=cdata/>
        }.into_view()
    };
    let pid = query.get_untracked().unwrap().id.expect("expected pid in query");
    let (refresh_get, refresh_set) = create_signal::<Option<()>>(None);
    view! {
        <AwaitWithError
            future=move||{
                let id = query.get_untracked().unwrap().id.expect("expected pid in query");
                webworks::get_post(id)
            }
            pending=move||view!{
                <p>"Loading Post"</p>
            }.into_view()
            ok_handler=move|ok| {
                let post_data = ok.clone();
                view!{
                    <DisplayPost
                        data=post_data
                        comment_button=false
                    />
                    <crate::comments::CommentForm post_id=pid on_posted=move|_|{
                        refresh_set.set(Some(()));
                    }/>
                    <InfiniteLoad
                        display=comment_display
                        loader=comments_loader
                        extra_data={pid}
                        refresh=refresh_get
                    />
                }.into_view()
            }
            err_handler=err_handler
        >
        </AwaitWithError>
    }
}
async fn comments_loader(pid: uuid::Uuid, toload: i32) -> Result<Vec<CommentData>, WebworksError> {
    webworks::load_comments(pid, toload).await
}

