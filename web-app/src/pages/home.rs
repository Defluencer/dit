use crate::app::AppRoute;
use crate::components::Navbar;

use yew::prelude::{classes, html, Component, ComponentLink, Html, ShouldRender};

/// The landing page.
pub struct Home {}

impl Component for Home {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let body_one = html! {
            <ybc::Container classes=classes!("has-text-centered") >
                <ybc::Title tag="h1" >
                {
                    "Build social media applications and websites on the web 3.0"
                }
                </ybc::Title>
                <ybc::Subtitle tag="h2" >
                {
                    "Defluencer is a protocol for decentralized social media.
                    Are you an influencer? Build your own website to show the world your awesome content!
                    Do you love wombats? Build a platform and agregate the world's content on the subject!
                    With immutable and interoperable content, your data cannot be changed and will follow you to any app or website built on the protocol."
                }
                </ybc::Subtitle>
                <ybc::ButtonRouter<AppRoute> route=AppRoute::Settings classes=classes!("is-primary") >
                    {"Get Started"}
                </ybc::ButtonRouter<AppRoute>>
            </ybc::Container>
        };

        let live_card = feature_card(
            "Live Streaming",
            "Custom resolution, quality and codecs.
            No central server, 100% peer to peer",
        );

        let chat_card = feature_card(
            "Live Chat",
            "Choose a display name and use your Ethereum public keys as your identity.",
        );

        let streaming_card = feature_card(
            "On Demand Streaming",
            "Live streams can be saved and viewed later or you can host your own videos.",
        );

        let blog_card = feature_card(
            "Blogs",
            "Twitter-style micro blog or for more lengthy posts, markdown files.",
        );

        let feed_card = feature_card(
            "Content Feed",
            "Organize your content into a multimedia feed that people can follow.",
        );

        let comments_card = feature_card(
            "Commentary",
            "Comment on any media or read what people you follow have to say.",
        );

        html! {
            <>
                <Navbar />
                <ybc::Hero classes=classes!("is-medium") body=body_one />
                <ybc::Section>
                    <ybc::Container>
                        <ybc::Columns classes=classes!("is-multiline") >
                            { live_card }
                            { chat_card }
                            { streaming_card }
                            { blog_card }
                            { feed_card }
                            { comments_card }
                        </ybc::Columns>
                    </ybc::Container>
                </ybc::Section>
                <ybc::Footer>
                    <ybc::Container>
                        <ybc::Columns>
                            <ybc::Column classes=classes!("is-half") >
                                <a href="https://github.com/SionoiS/dit">
                                    <span class="icon-text">
                                        <span> {"Source Code"} </span>
                                        <span class="icon"><i class="fab fa-github"></i></span>
                                    </span>
                                </a>
                            </ybc::Column>
                            <ybc::Column classes=classes!("is-half") >
                                <a href="https://bulma.io">
                                    <img src="https://bulma.io/images/made-with-bulma.png" alt="Made with Bulma" width="128" height="24" />
                                </a>
                            </ybc::Column>
                        </ybc::Columns>
                    </ybc::Container>
                </ybc::Footer>
            </>
        }
    }
}

fn feature_card(title: &str, text: &str) -> Html {
    html! {
        <ybc::Column classes=classes!("is-half", "is-flex") >
            <ybc::Card>
                <ybc::CardContent>
                    <ybc::Media>
                        <ybc::MediaContent>
                            <ybc::Title tag="h1" size=ybc::HeaderSize::Is4 > { title } </ybc::Title>
                        </ybc::MediaContent>
                    </ybc::Media>
                    <ybc::Content>
                        <ybc::Subtitle tag="div" > { text } </ybc::Subtitle>
                    </ybc::Content>
                </ybc::CardContent>
            </ybc::Card>
        </ybc::Column>
    }
}
