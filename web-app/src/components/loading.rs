use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

/// Loading indicator.
#[derive(Clone, Properties)]
pub struct Loading {}

impl Component for Loading {
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
        html! {
            <ybc::Box>
                <div>
                    { "Loading..." }
                </div>
                <progress class="progress is-info">
                    { "0%" }
                </progress>
            </ybc::Box>
        }
    }
}
