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
        if !Rc::ptr_eq(&self.comment, &props.comment) || !Rc::ptr_eq(&self.name, &props.name) {
            *self = props;

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <ybc::Box>
                <ybc::Media>
                    <ybc::MediaLeft>
                        { &self.name }
                    </ybc::MediaLeft>
                    <ybc::MediaContent>
                        { &self.comment.comment }
                    </ybc::MediaContent>
                </ybc::Media>
            </ybc::Box>
        }
    }
}
