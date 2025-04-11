use leptos::{component, create_node_ref, NodeRef, view, IntoView};
use crate::data::CommentData;

#[component]
pub fn DisplayComment(data: CommentData) -> impl IntoView {
    view!{
        <p>"DisplayComment"</p>
    }
}
#[component]
pub fn CommentForm() -> impl IntoView {
    let comment_content: NodeRef<leptos::html::Textarea> = create_node_ref();
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
                //on:click=on_click_post
                type="button">
                "Leave comment"
            </button>
            //</div>
        </div>
    }
}
