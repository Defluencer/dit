use crate::components::Navbar;
use crate::utils::LocalStorage;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender, classes};

/// Page with app settings and options.
pub struct Settings {
    link: ComponentLink<Self>,

    address: String,
    valid: bool,

    storage: LocalStorage,
}

pub enum Msg {
    Addrs(String),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub storage: LocalStorage,
}

impl Component for Settings {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props { storage } = props;

        let address = match storage.get_local_ipfs_addrs() {
            Some(addrs) => addrs,
            None => crate::utils::DEFAULT_URI.to_owned(),
        };

        let valid = reqwest::Url::parse(&address).is_ok();

        Self { link, address, valid, storage }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Addrs(msg) => self.addrs(msg),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let error_class = match self.valid {
            true => None,
            false => Some(classes!("is-danger")),
        };

        html! {
            <>
                <Navbar />
                <ybc::Section>
                    <ybc::Field label="IPFS API".to_owned() help="Refresh to apply changes".to_owned() help_classes=error_class.clone() >
                        <ybc::Control>
                            <ybc::Input classes=error_class name="ipfs_addrs" value=self.address.clone() update=self.link.callback(Msg::Addrs) placeholder=crate::utils::DEFAULT_URI />
                        </ybc::Control>
                    </ybc::Field>
                </ybc::Section>
            </>
        }
    }
}

impl Settings {
    fn addrs(&mut self, msg: String) -> bool {
        self.valid = reqwest::Url::parse(&msg).is_ok();

        if self.valid {
            self.storage.set_local_ipfs_addrs(&msg);
        }
        
        self.address = msg;

        true
    }
}
