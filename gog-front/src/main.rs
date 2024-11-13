#![allow(clippy::pedantic)]

mod webworks;
mod errors;
mod posts;

pub(crate) mod data;

use data::UserData;
use errors::{LoginError, PfpUploadError, RegisterError, UpdateUserError};
use leptos::leptos_dom::logging::{self, console_error};
use leptos::{component, create_resource, create_action, create_node_ref, event_target, event_target_value, expect_context, prelude::*, provide_context, spawn_local, with, CollectView, IntoView, NodeRef};
use leptos::view;
use leptos_router::Params;
use leptos_router::{use_navigate, use_query, NavigateOptions, Route, Router, Routes};
use leptos::logging::*;
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| view! { <App/> })
}


#[component]
fn NotFound()  -> impl IntoView {
    view!{
        <h1 style="text-align: center;">"Not Found"</h1>
    }
}

#[component]
fn App() -> impl IntoView {
    // let on_submit = move |ev: leptos::ev::SubmitEvent| {
    //     ev.prevent_default();
    // };
    provide_context(create_rw_signal::<Option<UserData>>(None));
    view!{
        <Router>
            <nav class="nav">
                <Navigation/>
            </nav>
            <main class="center">
                <div>
                    <Routes>
                            <Route path="/login" view=LoginForm/>
                            <Route path="/user" view=UserScreen/>
                            <Route path="/user/edit" view=EditUser/>
                            <Route path="/users" view=DisplayOtherUser/>
                            <Route path="/register" view=RegisterForm></Route>
                            <Route path="/posts" view=posts::Posts /> 
                            <Route path="*any" view=NotFound/>
                    </Routes>
                </div>
            </main>
        </Router>
    }
}

#[component]
fn Navigation() -> impl IntoView {
    use leptos_router::A;
    view!{
        <ul>
            <li style="float: right;"><A href="register">"Register"</A></li>
            <li style="float: right;"><A href="login">"Login"</A></li>
            <li><A href="user">"User"</A></li>
            <li><A href="posts">"Posts"</A></li>
        </ul>
    }
}






#[component]
fn RegisterForm() -> impl IntoView {
    use leptos_router::Form;

    let (login, set_login) = create_signal("".to_string());
    let (password, set_password) = create_signal("".to_string());
    let (rep_password, set_rep_password) = create_signal("".to_string());


    let (validation_msg , set_validation_msg) = create_signal("");
    let register_action = create_action(move|usr: &data::UserCreationData|{
        let data = usr.clone();
        async move { webworks::register(&data).await }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let login = login.get();
        let password = password.get();
        let rep = rep_password.get();

        if rep.ne(&password) {
            set_validation_msg.set("Password does not match");
            return;
        }

        register_action.dispatch(data::UserCreationData{ 
            login,
            password,
        });
        set_validation_msg.set("");
    };

    let result = register_action.value();

    let register_result = move || {
        result.with(|r| match &r {
            &Some(Ok(_)) => {
                let nav = leptos_router::use_navigate();
                nav("/login", NavigateOptions::default());
                view!{
                    <p>"Registered successfuly"</p>
                }.into_view()
            },
            &Some(Err(err)) => {
                match err {
                    RegisterError::ValidationError(body) => {
                        let reason = &body.reason;
                        let errors = &body.errors.iter()
                            .map(|(k, v)| {

                                let errs = v.iter()
                                    .map(|e| {
                                        view! {
                                            <li>{e.message.clone()}</li>
                                        }
                                    }).collect_view();

                                view!{
                                    {k}
                                    <ul>{errs}</ul>
                                }
                            }).collect_view();
                        view!{
                            <p>{reason}</p>
                            <p>"Reported errors:"</p>
                            <ul>
                                {errors}
                            </ul>
                        }.into_view()
                    },
                    RegisterError::Webworks { source } => {
                        logging::console_error(&source.to_string());
                        view!{
                            <p>"An error has occured"</p>
                        }.into_view()
                    }
                }.into_view()
            },
            None => view! {
                <p></p>
            }.into_view()
        })
    };

    let valid = move || {
        with!(move |password, rep_password| 
            if password.eq(rep_password) && !password.is_empty() {
                view!{
                    <span style="color: green;">"✔"</span>
                }
            } else {
                view!{
                    <span style="color: red;">"✖"</span>
                }
            }
        )
    };

    view! {
        <div>
            <h3 style="text-align: center;">"Register a new account"</h3>
            <Form method="GET" action="" class="formcenter"
                on:submit=on_submit>
                <label for="reg-login">"Login:"</label><br/>
                <input type="text" id="reg-login" 
                    on:input=move |ev| {
                        set_login.set(event_target_value(&ev));
                    }
                    />
                <br/>
                <label for="reg-password">"Password:"</label><br/>
                <input type="password" id="reg-password" 
                    on:input=move |ev| {
                       set_password.set(event_target_value(&ev)); 
                    }/>
                {valid}
                <br/>
                <label for="reg-rep-password">"Repeat password:"</label><br/>
                <input type="password" id="reg-rep-password" 
                    on:input=move |ev| {
                        set_rep_password.set(event_target_value(&ev));
                    }/>
                {valid}
                <br/>
                <input type="submit"/><br/>
            </Form>
            <p>{validation_msg}</p>
            <p>{register_result}</p>
        </div>
    }
}


#[component]
fn LoginForm(
) -> impl IntoView {

    use leptos_router::Form;
    let login: NodeRef<leptos::html::Input> = create_node_ref();
    let password: NodeRef<leptos::html::Input> = create_node_ref();

    let get_token_action = create_action(|input: &data::LoginData|{
        let input = input.clone();
        async move { webworks::get_token(&input).await }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.default_prevented();

        let login = login.get()
            .expect("<input id=\"login\"> should be mounted")
            .value();
        let password = password.get()
            .expect("<input id=\"password\"> should be mounted")
            .value();

        let data = data::LoginData{
            login,
            password
        };

        get_token_action.dispatch(data);
    };

    let pending = get_token_action.pending();
    let result = get_token_action.value();
    let logging_in = move || {
        pending.get().then_some("Logging in...")
    };

    let outcome = move || {
        result.with(|r| match &r {
            &Some(Ok(())) => {
                spawn_local(async move {
                    let nav = use_navigate();
                    nav("/user", NavigateOptions::default());
                });
                view!{
                    <p>"Logged in!"</p>
                }.into_any()
            },
            &Some(Err(err)) => {
                match err {
                    LoginError::IncorrectPassword => {
                        view!{
                            <p>"Password was incorrect"</p>
                        }
                    },
                    LoginError::NoSuchUser => {
                        view!{
                            <p>"No such user exists"</p>
                        }
                    },
                    LoginError::Webworks { source } => {
                        error!("webworks error {:?}", source);
                        view! {
                            <p>"An unknown error has occured"</p>
                        }
                    }
                }.into_any()
            },
            None => {
                view!{
                    <p>""</p>
                }.into_any()
            }
        })
    };
    view!{
        <h3 style="text-align: center;">"Log in to an account"</h3>
        <Form method="GET" action="" class="formcenter"
            on:submit=on_submit>
            <label for="login">Login</label><br/>
            <input id="login" type="text" node_ref=login/><br/>
            <label for="password">Password</label><br/>
            <input id="password" type="password" node_ref=password/><br/>
            <input type="submit"/>
        </Form>
        <p>{logging_in}</p>
        <p>{outcome}</p>
    }
}



#[component]
fn UserScreen() -> impl IntoView {
    use leptos::Await;
    view! {
        <Await
            future=webworks::get_user_data
            let:data
        >
            <DisplayUser user_data=data.clone()/>
        </Await>
    }
}

#[component]
fn EditUser() -> impl IntoView {
    let user_data_state = expect_context::<RwSignal<Option<UserData>>>();
    let Some(user_data) = user_data_state.get() else {
        let nav = use_navigate();
        nav("/user", NavigateOptions::default());
        return view! {}.into_view()
    };
    let (data, set_data) = create_signal(user_data);

    let edit_desc = move |ev: leptos::ev::Event| {
        set_data.update(|d| d.description = event_target_value(&ev));
    };
    let edit_gender = move|ev: leptos::ev::Event| {
        set_data.update(|d| d.gender = event_target_value(&ev));
    };

    let update_action = create_action(|input: &data::UserData|{
        let input = input.clone();
        async move { webworks::update_user_data(&input).await }
    });

    let pending = update_action.pending();
    let result = update_action.value();
    let outcome = move || {
        result.with(|r| {
            match r {
                Some(Ok(_)) => view!{
                    <p>"Updated user data!"</p>
                }.into_view(),
                Some(Err(e)) => match e {
                    UpdateUserError::ValidationError(ve) => {
                        let reason = &ve.reason;
                        let errors = &ve.errors.iter()
                            .map(|(k, v)| {

                                let errs = v.iter()
                                    .map(|e| {
                                        view! {
                                            <li>
                                                <div>
                                                    <p>
                                                    {e.message.clone()}
                                                    {e.params.min.map(|v| view!{<p>" min: "{v}</p>})}
                                                    {e.params.max.map(|v| view!{<p>" max: "{v}</p>})}
                                                    </p>
                                                </div>
                                            </li>
                                        }
                                    }).collect_view();

                                view!{
                                    <ul>{errs}</ul>
                                }
                            }).collect_view();
                        view!{
                            <p>{reason}</p>
                            <p>"Reported errors:"</p>
                            <ul>
                                {errors}
                            </ul>
                        }.into_view()
                    },
                    UpdateUserError::Webworks { source } => {
                        console_error(&source.to_string());
                        view! {
                            <p>"An unknown error has occured"</p>
                        }.into_view()
                    }    
                },
                _ => view!{<p/>}.into_view()
            }
        })
    };
    let updating = move || {
        if pending.get() {
            view!{ "Updating data..." }
        } else {
            view!{ "" }
        }
    };

    let on_save = move |_| {
        let user_data = data.get();
        update_action.dispatch(user_data);
    };

    let update_pfp_action = create_action(|file: &web_sys::File| {
        let input = file.clone();
        async move { 
            let mut rec = webworks::upload_new_pfp(input).await;
            rec.recv().await
        }
    });
    let pfp_pending = update_pfp_action.pending();
    let update_pfp_value = update_pfp_action.value();
    let pfp_outcome = move || {
        update_pfp_value.with(|v| {
            let Some(v) = v else {
                return view!{}.into_view();
            };
            match v {
                None => view!{}.into_view(),
                Some(r) => match r {
                    Ok(_) => {
                        //leptos::document().location().unwrap().reload().unwrap();
                        view!{<p>"Uploaded!"</p>}.into_view()
                    },
                    Err(e) => match e {
                        PfpUploadError::Rejected { reason } => {
                            view!{<p>"File rejected: " {reason}</p>}.into_view()
                        },
                        PfpUploadError::Websys { js_value } => {
                            let v = js_value.as_string().unwrap_or_default();
                            console_error(&v);                            
                            view!{<p>"An error has occured"</p>}.into_view()
                        },
                        _ => {
                            view!{<p>"An error has occured"</p>}.into_view()
                        }
                    }
                }
            }
        })
    };

    let on_input_image = move |ev: web_sys::Event| {
        use leptos::web_sys;
        let target: web_sys::HtmlInputElement = event_target(&ev);
        let file_list = target.files().unwrap();
        let file = file_list.get(0).unwrap();
        update_pfp_action.dispatch(file);
    };

    view! {
        <div>
            <h1>"Editing profile of user: " {data.get().login}</h1>
            <table style="width:100%;table-layout:fixed;border: 1px dotted white; padding:10px;" >
                <tr>
                    <td>
                        <label for="genderinput">"Gender: "</label>
                        <input type="text" 
                            name="gender" 
                            id="generinput"
                            list="genders"
                            on:input=edit_gender
                            prop:value={data.get().gender}/>
                        <datalist id="genders">
                            <option value="male"/>
                            <option value="female"/>
                        </datalist>
                        <p>"Edit description:"</p>
                        <textarea type="text" wrap="hard" rows="20"
                        on:input=edit_desc
                        prop:value={data.get().description}
                        style="width:100%;"
                        /><br/>
                    </td>
                    <td>
                        <div style="text-align: center">
                        <img src={webworks::get_pfp_url_for_login(&data.get().login)} 
                            alt="User profile picture"
                            style="width:200px;height:200px;"
                            />
                        <br/>

                        <label for="pfp">
                            "Choose a new profile picture"
                        </label>
                        <br/>
                        <input type="file" 
                            name="pfp-file" 
                            id="pfp"
                            accept="image/jpeg"
                            on:change=on_input_image
                        />
                        { move || {
                            pfp_pending.get().then(|| view!{
                                <p>"Uploading file..."</p>
                            })
                        }}
                        {pfp_outcome}
                        </div>
                    </td>
                </tr>
            </table>
            <button
                on:click=on_save
                type="button">
                "Save"
            </button>
            <button
                on:click=move |ev| {
                    let nav = use_navigate();
                    nav("/user", NavigateOptions::default());
                }
            >
                "Cancel"
            </button><br/>
            <p>{updating}</p>
            {outcome}
        </div>
    }.into_view()
}

#[component]
fn DisplayUser(user_data: Option<data::UserData>) -> impl IntoView {
    let Some(user_data) = user_data else {
        return view! {
            <h1 style="text-align:center;">"User not logged in"</h1>
        }.into_view()
    };
    let logout_action = create_action(|_: &()| {
        async move { webworks::logout_user().await }        
    });
    let data = expect_context::<RwSignal<Option<UserData>>>();
    data.set(Some(user_data.clone()));
    view! {
        <div>
        <table style="width:100%;table-layout:fixed;">
            <tr>
                <td style="border: 1px dotted white; padding:10px;">
                    <h1>{&user_data.login}</h1>
                    <p>"Joined: " {
                        format!("{}", &user_data.created.unwrap_or_default().format("%Y-%m-%d"))
                    }</p>
                    <p>"Gender: " {&user_data.gender}</p>
                    <p>"Description: " {user_data.description}</p>
                </td>
                <td style="text-align: right">
                    <div>
                    <img src={webworks::get_pfp_url_for_login(&user_data.login)} 
                        alt="User profile picture"
                        style="width:200px;height:200px;"/>
                    </div>
                </td>
            </tr>
        </table>
        <button
            on:click=move|_| {
                let nav = use_navigate();
                nav("user/edit", NavigateOptions { 
                    resolve: true,
                    ..Default::default() 
                });
            }>
            "Edit profile"
        </button>
        <button
            on:click=move|_| {
                logout_action.dispatch(());
                let nav = use_navigate();
                nav("/login", NavigateOptions::default());
                data.set(None);
            }>
            "Log out"
        </button>
        </div>
    }.into_view()

}

#[component]
fn DisplayOtherUser() -> impl IntoView {
    use data::UserProfileQuery;
    let query = use_query::<UserProfileQuery>();

    if query.with(|q| q.is_err()){
        return view!{<NotFound/>};
    }

    let (get_un, _) = create_signal(query.with(|q|q.clone().unwrap()));

    let user_data = create_resource(move|| get_un.get(), 
    |ud| async move { 
            let res = webworks::get_user_profile(ud).await;
            res.ok()
        });
    let display = move|data: UserData| {
        view!{
            <div>
            <table style="width:100%;table-layout:fixed;">
                <tr>
                    <td style="border: 1px dotted white; padding:10px;">
                        <h1>{data.login.clone()}</h1>
                        <p>"Joined: " {
                            format!("{}", data.created.unwrap_or_default().format("%Y-%m-%d"))
                        }</p>
                        <p>"Gender: " {data.gender}</p>
                        <p>"Description: " {data.description}</p>
                    </td>
                    <td style="text-align: right">
                        <div>
                        <img src={webworks::get_pfp_url_for_login(&data.login)} 
                            alt="User profile picture"
                            style="width:200px;height:200px;"/>
                        </div>
                    </td>
                </tr>
            </table>
            </div>
        }
    };

    use leptos::Suspense;
    view!{
        <Suspense
            fallback=move||{view!{}}
        >
            {move|| {
                match user_data.get() {
                    Some(ud) => match ud {
                        Some(ud) => display(ud).into_view(),
                        None => view!{<NotFound/>}
                    },
                    None => view!{ 
                        <p>"Could not load user profile"</p>
                    }.into_view()
                }
            }}
        </Suspense>
    }
}