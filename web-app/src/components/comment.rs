use std::rc::Rc;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use linked_data::signature::SignedMessage;

#[derive(Clone, Properties)]
pub struct Comment {
    pub signed_comment: Rc<SignedMessage<linked_data::comments::Comment>>,
}

//TODO find a way to get a name for each comments

impl Component for Comment {
    type Message = ();
    type Properties = Self;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.signed_comment != props.signed_comment {
            *self = props;

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="=comment">
                { &self.signed_comment.data.comment }
            </div>
        }
    }
}
