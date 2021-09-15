use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};

/// Error indicator.
#[derive(Clone, Properties)]
pub struct Error {}

impl Component for Error {
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
                {
                    "An Error was encounted.
                    Please verify your connection to the Ethereum and IPFS networks"
                }
                </ybc::Box>
            </ybc::Container>
        }
    }
}
