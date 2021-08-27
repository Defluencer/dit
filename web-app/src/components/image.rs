use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use cid::Cid;

/// Image from IPFS.
#[derive(Clone, Properties)]
pub struct Image {
    pub image_cid: Cid,
}

impl Component for Image {
    type Message = ();
    type Properties = Self;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if props.image_cid != self.image_cid {
            *self = props;

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <img src=format!("ipfs://{}", self.image_cid.to_string()) alt="This image require IPFS native browser" />
        }
    }
}
