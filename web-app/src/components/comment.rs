use std::rc::Rc;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

#[derive(Clone, Properties)]
pub struct Comment {
    pub name: Rc<str>,
    pub comment: Rc<linked_data::comments::Comment>,
}

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
        if self.comment != props.comment {
            *self = props;

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="comment">
                <div class="comment_name"> { &self.name } </div>
                <div class="comment_text"> { &self.comment.comment } </div>
            </div>
        }
    }
}
