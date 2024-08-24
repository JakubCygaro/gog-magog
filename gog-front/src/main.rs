mod webworks;
mod errors;
pub(crate) mod data;

use errors::RegisterError;
use leptos::leptos_dom::logging;
use leptos::{component, create_action, create_node_ref, create_resource, prelude::*, spawn_local, Callback, Children, ChildrenFn, CollectView, Fragment, IntoView, NodeRef};
use leptos::view;
use leptos_router::{use_navigate, NavigateOptions, Route, Router, Routes};
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

    let login: NodeRef<leptos::html::Input> = create_node_ref();
    let password: NodeRef<leptos::html::Input> = create_node_ref();
    let rep_password: NodeRef<leptos::html::Input> = create_node_ref();

    let (validation_msg , set_validation_msg) = create_signal("");
    let (valid, set_valid) = create_signal(false);
    let register_action = create_action(move|usr: &data::UserCreationData|{
        let data = usr.clone();
        async move { webworks::register(&data).await }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let login = login.get().unwrap().value();
        let password = password.get().unwrap().value();
        let rep = rep_password.get().unwrap().value();

        if rep.ne(&password) {
            set_validation_msg.set("Password does not match");
            set_valid.set(false);
            return;
        }

        register_action.dispatch(data::UserCreationData{ 
            login: login,
            password: password,
        });
        set_validation_msg.set("");
        set_valid.set(true);

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

    let validation = move || {
        valid.with(|v| {
            if *v {
                view! {
                    <span class="color: green;">"✔"</span>
                }
            } else {
                view! {
                    <span class="color: red;">"✖"</span>
                }
            }.into_any()
        })
    };

    view! {
        <div>
            <h3 style="text-align: center;">"Register a new account"</h3>
            <Form method="GET" action="" class="formcenter"
                on:submit=on_submit>
                <label for="reg-login">"Login:"</label><br/>
                <input type="text" id="reg-login" node_ref=login/><br/>
                <label for="reg-password">"Password:"</label><br/>
                <input type="password" id="reg-password" node_ref=password/>
                {validation}
                <br/>
                <label for="reg-rep-password">"Repeat password:"</label><br/>
                <input type="password" id="reg-rep-password" node_ref=rep_password/>
                {validation}
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
        logging::console_log("called");
    };

    //let subbmited = get_token_action.input();
    let pending = get_token_action.pending();
    let result = get_token_action.value();
    let penis = move || {
        pending.get().then(move|| {"Logging in..."})
            // .or_else(move|| {
            //     let navigate =use_navigate();
            //     navigate("/user", NavigateOptions::default());
            //     Some("Logging in...")
            // })
    };

    let outcome = move || {
        return result.with(|r| match &r {
            &Some(Ok(())) => {

                spawn_local(async move {
                    let nav = use_navigate();
                    nav("/user", NavigateOptions::default());
                });

                view!{"Logged in!"}
            },
            &Some(Err(e)) => {
                logging::console_error(&format!("{:?}", &e));
                view!{"Failed to log in!"}
            },
            _=> {
                view!{""}
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
        <p>{penis}</p>
        <p>{outcome}</p>
    }
}



#[component]
fn UserScreen() -> impl IntoView {
    let user_data = create_resource(|| (), 
        |_| async move {
            webworks::get_user_data().await
        });
    
    let display_user = move || {
        let Some(data) = user_data.get() else {
            return view!{
                <h1>"Fetching Data..."</h1>
            };
        };
        return match data {
            Some(user_data) => {
                view!{
                    <h1>{user_data.login}</h1>
                }
            },
            None => {
                view!{
                    <h1>"You are not logged in"</h1>
                }
            }
        }
        
    };
    use leptos::{Await, Show};
    view! {
        //{display_user}
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