use gog_commons::data_structures::CommentCreationData;
use leptos::{component, create_action, create_node_ref, create_signal, view, Callable, IntoView, NodeRef, SignalGet, SignalWith};
use crate::webworks;
use crate::data::CommentData;

#[component]
pub fn DisplayComment(data: CommentData) -> impl IntoView {
    let (get_data, set_data) = create_signal(data);
    view!{
        <div class="flex-container comment-section">
            <div class="flex-column">
                //<a
                //    href=move||{format!("users?name={}", get_data.get().login)}
                //    class="user-profile-link"
                //>{
                //    move||{get_data.get().login}
                //}</a>
                //<img
                //    src=move|| { webworks::get_pfp_url_for_login(get_data.get().login.as_str())}
                //    height="100"
                //    width="100"
                //    style="padding: 10px;display: block;
                //                    margin-left: auto;
                //                    margin-right: auto;"
                ///>
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
#[component]
pub fn CommentForm(post_id: uuid::Uuid, #[prop(into)] on_posted: leptos::Callback<()>) -> impl IntoView {
    let comment_content: NodeRef<leptos::html::Textarea> = create_node_ref();
    let comment_action = create_action(|comment_data: &CommentCreationData|{
        let comment_data = comment_data.to_owned();
        async move {
            webworks::leave_comment(comment_data).await
        }
    });
    let on_click_leave_comment = move|_ev: _| {
        let ccdata = CommentCreationData {
            content: comment_content.get().unwrap().value(),
            post_id
        };
        comment_action.dispatch(ccdata);
        on_posted.call(());
    };
    let comment_action_pending = comment_action.pending();
    let comment_action_value = comment_action.value();
    let comment_action_outcome = move|| {
        comment_action_value.with(|out|{
            match out {
                None => view!{}.into_view(),
                Some(res) => match res {
                    Ok(_) => view!{}.into_view(),
                    Err(e) => view!{<p>"Error"</p>}.into_view()
                }
            }
        })
    };
    view!{
        <div class="flex-column comment-section">
            //<div class="flex-column">
            //    <p
            //        style="padding:0;margin:0;text-align:center;"
            //    >
            //    {move||{get.get().unwrap().login}}
            //    </p>
            //    <img
            //        src=move|| { webworks::get_pfp_url_for_login(&get.get().unwrap().login)}
            //        height="100"
            //        width="100"
            //        style="padding: 10px;"
            //    />
            //    <button
            //        style="padding: 5px; "
            //        on:click=on_click_post
            //        type="button">
            //        "Post"
            //    </button>
            //
            <p>"Write a comment:"</p>
            <textarea type="text" wrap="hard" rows="5"
                        class="post-textbox"
                        prop:value="Comment text"
                        maxlength="300"
                        node_ref=comment_content
                        />
            <button
                style="padding: 5px; "
                on:click=on_click_leave_comment
                type="button">
                "Leave comment"
            </button>
            {comment_action_outcome}
            //</div>
        </div>
    }
}
