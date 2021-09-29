use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};

use cid::Cid;

#[derive(Clone, Properties)]
pub struct ExploreCid {
    pub cid: Cid,
}

impl Component for ExploreCid {
    type Message = ();
    type Properties = Self;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let link = format!("https://webui.ipfs.io/#/explore/{}", self.cid.to_string());

        html! {
            <ybc::ButtonAnchor classes=classes!("is-small", "is-outlined", "is-primary") href=link >
                { "Explore" }
            </ybc::ButtonAnchor>
        }
    }
}
