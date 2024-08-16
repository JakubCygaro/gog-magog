
use leptos::ev::{SubmitEvent};
use leptos::leptos_dom::logging;
use leptos::svg::view;
use leptos::{component, create_action, create_node_ref, create_resource, prelude::*, spawn_local, Callback, Children, ChildrenFn, CollectView, Fragment, IntoView, NodeRef};
use leptos::view;
use leptos_router::{use_navigate, NavigateOptions, Route, RouteProps, Router, RouterProps, Routes, RoutesProps};
use anyhow::{anyhow, Result};
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



    let resp = Request::post("http://localhost:8081/user/token")
        .header("Content-Type", "application/json")
        .credentials(leptos::web_sys::RequestCredentials::Include)
        .body(body)
        .or_else(|e| Err(anyhow!(e)))?
        .send()
        .await
        .or_else(|e| Err(anyhow!(e)))?;


    Ok(())
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
            <input id="password" type="text" node_ref=password/><br/>
            <input type="submit"/>
        </Form>
        <p>{penis}</p>
        <p>{outcome}</p>
    }
}

#[derive(Clone, serde::Deserialize, Serialize)]
struct UserData{
    pub login: String
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

    view! {
        {display_user}
    }
}