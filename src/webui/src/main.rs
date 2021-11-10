use wasm_bindgen::JsValue;
use web_sys::console;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

enum Msg {
    Play(String),
    Stop,
}

struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Stop => {
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            },
            _ => panic!(),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <button onclick=self.link.callback(|_| Msg::Stop)>{ "Stop" }</button>
            </div>
        }
    }
}

async fn fetch_stuff() {
    let res = reqwest::Client::new()
        .get("http://localhost:3030/api/cassettes")
        .send()
        .await
        .unwrap()
        .text().await.unwrap();
    console::log_1(&res.into());
}

fn main() {
    spawn_local(fetch_stuff());
    yew::start_app::<Model>();
}
