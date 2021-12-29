use std::collections::HashMap;
use std::fmt::Write;

use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use kasetophono::Cassette;

enum Msg {
    Play(Uuid),
    Cassettes(HashMap<Uuid, Cassette>),
    Stop,
}

struct Model {
    link: ComponentLink<Self>,
    cassettes: Vec<(Uuid, Cassette)>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link_fut = link.clone();
        spawn_local(async move {
            let cassettes = fetch_cassettes().await;
            link_fut.send_message(Msg::Cassettes(cassettes));
        });
        Self {
            link,
            cassettes: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Cassettes(cassettes) => {
                self.cassettes = cassettes.into_iter().collect();
                self.cassettes
                    .sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));
                true
            }
            Msg::Play(uuid) => {
                spawn_local(play_cassette(uuid));
                false
            }
            Msg::Stop => {
                spawn_local(stop());
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <img class="title" src="https://3.bp.blogspot.com/-xTPGTrjKbcc/WbGOKSAWMQI/AAAAAAAAPQM/UY9fma6zC9kpAWKK8Vd1xbJhKVxiDHh2wCK4BGAYYCw/s600/kasetophono.png"/>
                <button class="stop" onclick=self.link.callback(|_| Msg::Stop)>{ "Stop" }</button>
                <table>
                    <thead>
                        <tr>
                            <th></th>
                            <th>{"Title"}</th>
                            <th>{"Created At"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {for self.cassettes.iter().map(|&(uuid, ref cassette)| {
                            html! {
                                <tr key={uuid.to_string()}>
                                    <td><button class="play" onclick=self.link.clone().callback(move |_| Msg::Play(uuid))>{"Play"}</button></td>
                                    <td>{&cassette.name}</td>
                                    <td>{&cassette.created_at[0..10]}</td>
                                </tr>
                            }
                        })}
                    </tbody>
                </table>
            </div>
        }
    }
}

async fn stop() {
    let window = web_sys::window().unwrap();
    let mut path = window.location().origin().unwrap();
    path.push_str("/api/stop");

    reqwest::Client::new()
        .get(path)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
}

async fn play_cassette(uuid: Uuid) {
    let window = web_sys::window().unwrap();
    let mut path = window.location().origin().unwrap();
    write!(path, "/api/play/{}", uuid).expect("infallible");

    reqwest::Client::new()
        .get(path)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
}

async fn fetch_cassettes() -> HashMap<Uuid, Cassette> {
    let window = web_sys::window().unwrap();
    let mut path = window.location().origin().unwrap();
    path.push_str("/api/cassettes");

    let res = reqwest::Client::new()
        .get(path)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    serde_json::from_str(&res).unwrap()
}

fn main() {
    yew::start_app::<Model>();
}
