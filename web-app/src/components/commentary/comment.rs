use std::rc::Rc;

use crate::components::ExploreCid;

use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};

use cid::Cid;

#[derive(Clone, Properties)]
pub struct Comment {
    pub cid: Cid,
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
        let dt = crate::utils::timestamp_to_datetime(self.comment.timestamp);

        html! {
            <ybc::Box>
                <ybc::Media>
                    <ybc::MediaLeft>
                        <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-user"></i></span>
                                <span> { &self.name } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-clock"></i></span>
                                <span> { dt } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <ExploreCid cid=self.cid />
                        </ybc::Block>
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
