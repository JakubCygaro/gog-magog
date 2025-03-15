use super::loader::*;
use leptos_router::{use_navigate, use_query, NavigateOptions, Route, Router, Routes};
use std::future::{self, pending};
use std::pin::Pin;
use std::str::FromStr;

use leptos::html::Output;
use leptos::svg::filter;
use leptos::{create_action, document,  Callable, Effect, Serializable};
use leptos::{component, IntoView, Await, Suspense, ErrorBoundary, view, prelude::*, expect_context, create_resource, Show, NodeRef, create_node_ref, For};
use web_sys::wasm_bindgen::{self, JsCast};
use web_sys::{js_sys, HtmlBodyElement};
use crate::errors::{self, CreatePostError, WebworksError};
use crate::webworks::{PostsFilter, WebworksResult};
use leptos::logging::{log, warn, error, debug_warn};
use super::data::*;
use super::webworks;

#[component]
pub fn AwaitWithError<T, Fut, FF, E, P, IV, EH, OH>(
    future: FF,
    #[prop(optional)]
    pending: Option<P>,
    ok_handler: OH,
    err_handler: EH,
    ) -> impl IntoView
where
    IV: IntoView,
    T: Serializable + 'static,
    E: std::error::Error + 'static  ,
    Fut: std::future::Future<Output = Result<T, E>> + 'static,
    FF: Fn() -> Fut + 'static,
    P: Fn() -> IV + 'static,
    EH: Fn(&E) -> IV + 'static,
    OH: Fn(&T) -> IV + 'static,
{
    let action = leptos::create_action(move |_:&()| {
        future()
    });

    let result = action.value();
    let pen = action.pending();
    action.dispatch(());
    view!{
        {
            move ||{
                if let Some(p) = &pending {
                    pen.get().then(p)
                } else {
                    None
                }
            }
        }
        {
            move||{
                result.with(|r| {
                    match r {
                        None => view!{}.into_view(),
                        Some(Err(e)) => err_handler(e).into_view(),
                        Some(Ok(ok)) => ok_handler(ok).into_view(),
                    }
                })
            }
        }

    }.into_view()
}
