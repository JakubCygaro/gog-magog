mod webworks;
mod errors;
pub(crate) mod data;

use errors::{LoginError, RegisterError};
use leptos::leptos_dom::logging;
use leptos::svg::view;
use leptos::{component, create_action, create_node_ref, create_resource, event_target_value, prelude::*, spawn_local, with, Callback, Children, ChildrenFn, CollectView, Fragment, IntoView, NodeRef};
use leptos::view;
use leptos_router::{use_navigate, NavigateOptions, Route, Router, Routes};
use leptos::logging::*;
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| view! { <App/> })
}

#[component]
fn App() -> impl IntoView {
    // let on_submit = move |ev: leptos::ev::SubmitEvent| {
    //     ev.prevent_default();
    // };
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
                            <Route path="/register" view=RegisterForm/>
                            <Route path="*any" view=move ||{
                                view!{
                                    <h1 style="text-align: center;">"Not Found"</h1>
                                }
                            }/>
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
            <li><A href="register">"Register"</A></li>
            <li><A href="login">"Login"</A></li>
            <li><A href="user">"User"</A></li>
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
            login: login,
            password: password,
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
                }.into_any()
            },
            &Some(Err(err)) => {
                match err {
                    RegisterError::UserAlreadyExists => view!{
                        <p>"User already exists"</p>
                    }.into_any(),
                    RegisterError::ServerError{status} => {
                        logging::console_error(status);
                        view! {
                            <p>"A server side has error occured"</p>
                        }.into_any()
                    },
                    RegisterError::GlooError { err } => {
                        logging::console_error(&err.to_string());
                        view!{
                            <p>"An error has occured"</p>
                        }.into_any()
                    },
                    RegisterError::Unknown { msg } => {
                        logging::console_error(msg);
                        view!{
                            <p>"An unknown has error occured"</p>
                        }.into_any()
                    }
                }
            },
            None => view! {
                <p></p>
            }.into_any()
        })
    };

    let valid = move || {
        with!(move |password, rep_password| 
            if password.eq(rep_password) && password.len() > 0 {
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
            login: login,
            password: password
        };

        get_token_action.dispatch(data);
    };

    let pending = get_token_action.pending();
    let result = get_token_action.value();
    let logging_in = move || {
        pending.get().then(move|| {"Logging in..."})
    };

    let outcome = move || {
        return result.with(|r| match &r {
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
                    LoginError::ServerError { status } => {
                        error!("Server error: {}", status);
                        view!{
                            <p>"A server side error has occured"</p>
                        }
                    },
                    LoginError::GlooError { err } => {
                        error!("gloo_net error: {:?}", err);
                        view!{
                            <p>"An error has occured"</p>
                        }
                    },
                    LoginError::Unknown { msg } => {
                        error!("gloo_net error: {:?}", msg);
                        view!{
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
        });
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
            future=|| webworks::get_user_data()
            let:data
        >
            <DisplayUser user_data=data.clone()/>
        </Await>
    }
}

#[component]
fn DisplayUser(user_data: Option<data::UserData>) -> impl IntoView {
    let Some(user_data) = user_data else {
        return view! {
            <h1>"User not logged in"</h1>
        }.into_view()
    };
    view! {
        <h1>"Username: " {user_data.login}</h1>
        <p>"Description: " {user_data.description}</p>
    }.into_view()

}