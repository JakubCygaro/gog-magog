use std::result;

use gloo_net::http::Request;
use leptos::ev::{SubmitEvent};
use leptos::leptos_dom::logging;
use leptos::svg::view;
use leptos::{component, create_action, create_node_ref, create_resource, prelude::*, spawn_local, Callback, Children, ChildrenFn, CollectView, Fragment, IntoView, NodeRef};
use leptos::view;
use leptos_router::{use_navigate, NavigateOptions, Route, RouteProps, Router, RouterProps, Routes, RoutesProps};
use anyhow::{anyhow, Result};

const URL_BASE: &'static str = "http://localhost:8081/";

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
            <nav>
                <Navigation/>
            </nav>
            <main>
                <Routes>
                        <Route path="/login" view=LoginForm/>
                        <Route path="/user" view=UserScreen/>
                        <Route path="/register" view=RegisterForm/>
                        <Route path="*any" view=move ||{
                            view!{
                                <p>"Not Found"</p>
                            }
                        }/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn Navigation() -> impl IntoView {
    use leptos_router::A;
    view!{
        <A href="register">"Register"</A>
        <A href="login">"Login"</A>
        <A href="user">"User"</A>
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
struct LoginData {
    login: String,
    password: String
}

async fn get_token(data: &LoginData) -> Result<(), anyhow::Error> {
    use gloo_net::http::Request;

    let body = serde_json::to_string(data)?;



    let resp = Request::post(&(URL_BASE.to_owned() + "user/token"))
        .header("Content-Type", "application/json")
        .credentials(leptos::web_sys::RequestCredentials::Include)
        .body(body)
        .or_else(|e| Err(anyhow!(e)))?
        .send()
        .await
        .or_else(|e| Err(anyhow!(e)))?;


    Ok(())
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct UserCreationData {
    login: String,
    password: String,
}


async fn register(user_creation: &UserCreationData) -> Result<(), anyhow::Error> {
    let body = serde_json::to_string(&user_creation).unwrap();
    let resp = Request::post(&(URL_BASE.to_owned()+ "user/create"))
        .header("Content-Type", "application/json")
        .body(&body)
        .or_else(|e| Err(e))?
        .send()
        .await;
    match resp {
        Ok(_) => Ok(()),
        _ => Err(anyhow!("failed to register"))
    }
        
}

#[component]
fn RegisterForm() -> impl IntoView {
    use leptos_router::Form;

    let login: NodeRef<leptos::html::Input> = create_node_ref();
    let password: NodeRef<leptos::html::Input> = create_node_ref();
    let rep_password: NodeRef<leptos::html::Input> = create_node_ref();

    let (msg , set_msg) = create_signal("");
    
    let register_action = create_action(move|usr: &UserCreationData|{
        let data = usr.clone();
        async move { register(&data).await }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let login = login.get().unwrap().value();
        let password = password.get().unwrap().value();
        let rep = rep_password.get().unwrap().value();

        if rep.ne(&password) {
            set_msg.set("Password incorrect");
            return;
        }

        register_action.dispatch(UserCreationData{ 
            login: login,
            password: password,
        });
        set_msg.set("");

    };

    let result = register_action.value();

    let register_result = move || {
        result.with(|r| match &r {
            &Some(Ok(_)) => {
                let nav = leptos_router::use_navigate();
                nav("/login", NavigateOptions::default());
                move || {
                    "Registered successfuly"
                }
            },
            &Some(Err(_)) => {
                move || {
                    "Could not register user"
                }
            },
            _ => move || { "An error occured" }
        })
    };

    view! {
        <Form method="GET" action=""
            on:submit=on_submit>
            <label for="reg-login">"Login:"</label><br/>
            <input type="text" id="reg-login" node_ref=login/><br/>
            <label for="reg-password">"Password:"</label><br/>
            <input type="password" id="reg-password" node_ref=password/><br/>
            <label for="reg-rep-password">"Repeat password:"</label><br/>
            <input type="password" id="reg-rep-password" node_ref=rep_password/><br/>
            <input type="submit"/><br/>
        </Form>
        <p>{msg}</p>
        <p>{register_result}</p>
    }
}


#[component]
fn LoginForm(
) -> impl IntoView {

    use leptos_router::Form;
    let login: NodeRef<leptos::html::Input> = create_node_ref();
    let password: NodeRef<leptos::html::Input> = create_node_ref();

    let get_token_action = create_action(|input: &LoginData|{
        let input = input.clone();
        async move { get_token(&input).await }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.default_prevented();

        let login = login.get()
            .expect("<input id=\"login\"> should be mounted")
            .value();
        let password = password.get()
            .expect("<input id=\"password\"> should be mounted")
            .value();

        let data = LoginData{
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
        <Form method="GET" action=""
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

#[derive(Clone, serde::Deserialize, Serialize)]
pub struct UserData {
    pub login: String,
    pub id: String,
    pub description: String,
}


async fn get_user_data() -> Option<UserData> {
    use gloo_net::http::Request;

    let response = Request::get("http://localhost:8081/user/data")
        .credentials(leptos::web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .send()
        .await
        .or_else(|e| Err(anyhow!(e)));

    let Ok(response) = response else {
        return None;
    };

    let data = response.json::<UserData>().await;

    match data {
        Ok(d) => Some(d),
        Err(_) => None
    }
        
}

#[component]
fn UserScreen() -> impl IntoView {
    let user_data = create_resource(|| (), 
        |_| async move {
            get_user_data().await
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
            future=|| get_user_data()
            let:data
        >
            <DisplayUser user_data=data.clone()/>
        </Await>
    }
}

#[component]
fn DisplayUser(user_data: Option<UserData>) -> impl IntoView {
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