use std::rc::Rc;

use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};

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
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-user"></i></span>
                            <span> { &self.name } </span>
                        </span>
                    </ybc::MediaLeft>
                    <ybc::MediaContent>
                        <ybc::Content classes=classes!("has-text-centered") >
                            { &self.comment.comment }
                        </ybc::Content>
                    </ybc::MediaContent>
                </ybc::Media>
            </ybc::Box>
        }
    }
}
