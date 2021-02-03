use crate::components::Navbar;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use cid::Cid;

#[derive(Properties, Clone)]
pub struct Defluencer {
    pub beacon_cid: Cid,
}

impl Component for Defluencer {
    type Message = ();
    type Properties = Self;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <Navbar beacon_cid=self.beacon_cid />
                <div class="center_text"> {"Channel Page -> W.I.P."} </div>
            </>
        }
    }
}
