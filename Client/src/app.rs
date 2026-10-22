use std::time::Duration;
use iced;
use iced::{widget, Task};
use iced::widget::stack;

use crate::state::{View, RMessage, UXD};
use crate::login::LoginView;
use crate::main_view::MainView;
use crate::popup::BasePopup;

pub struct RIMessage {
    view: View,
    uxd: UXD,
    login_view: LoginView,
    popup_view: BasePopup,
    main_view: MainView
}
impl Default for RIMessage {
    fn default() -> Self {
        Self {
            view: View::Login,
            uxd: UXD::default(),
            login_view: LoginView::new(UXD::default()),
            popup_view: BasePopup::new(UXD::default()),
            main_view: MainView::new(UXD::default())
        }
    }
}

impl RIMessage {
    pub fn new(uxd: UXD) -> Self {
        Self { 
            view: View::Login, 
            uxd: uxd.clone(), 
            login_view: LoginView::new(uxd.clone()), 
            popup_view: BasePopup::new(uxd.clone()), 
            main_view: MainView::new(uxd.clone())
        }
    }
    pub fn view(&self) -> iced::Element<'_, RMessage> {
        stack![
            widget::space().width(iced::Fill).height(iced::Fill),
            widget::container(
                match self.view {
                    View::Login => self.login_view.view(),
                    View::Main => self.main_view.view()
                }
            ),
            self.popup_view.view(),
        ].into()

    }
    pub fn update(&mut self, msg: RMessage) -> iced::Task<RMessage> {
        match msg {
            RMessage::Sync => {
                // let mut s = String::new();
                // BufReader::new(&self.tcp).read_line(&mut s).unwrap();
                // self.ret = s;
                self.main_view.sync();
                Task::none()
            }
            RMessage::LoginView(m) => self.login_view.update(m),
            RMessage::PopupView(m) => self.popup_view.update(m),
            RMessage::Login(u) => {
                self.view = View::Main;
                self.main_view.req_stream = Some(self.login_view.login_stream.take().unwrap());
                self.main_view.user = Some(u);
                self.main_view.token = self.login_view.token.clone();
                dbg!(&self.main_view);
                self.main_view.try_connect_push_stream()
            }
            RMessage::MainView(m) => self.main_view.update(m),
            RMessage::Exit => iced::exit(),
            RMessage::UserCard(m) => self.main_view.msg_user_card(m),
            RMessage::TabView(m) => self.main_view.msg_tab_view(m),
            
        }
    }
    pub fn subscription(&self) -> iced::Subscription<RMessage> {
        iced::time::every(Duration::from_millis(500)).map(|_| RMessage::Sync)
    }
}