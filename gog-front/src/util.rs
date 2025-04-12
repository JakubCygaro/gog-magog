
use leptos::Serializable;
use leptos::{component, IntoView, view, prelude::*};

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
