use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};

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
            <ybc::Container classes=classes!("has-text-centered") >
                <ybc::Box>
                    <div>
                        { "Searching the decentralized web. Please wait..." }
                    </div>
                    <progress class="progress is-primary is-small">
                        { "0%" }
                    </progress>
                </ybc::Box>
            </ybc::Container>
        }
    }
}
