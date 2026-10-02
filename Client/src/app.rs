use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use iced;
use iced::{widget, Task};
use iced::widget::stack;
use iced_aw;

use crate::state::{View, RMessage, UXD};
use crate::login::LoginView;
use crate::main_view::MainView;
use crate::popup::BasePopup;

pub struct RIMessage {
    ret: String,
    view: View,
    uxd: UXD,
    login_view: LoginView,
    popup_view: BasePopup,
    main_view: MainView
}
impl Default for RIMessage {
    fn default() -> Self {
        // let mut revc = TcpStream::connect("127.0.0.1:7878").unwrap();
        // let rq = NMessage {
        //     sender: 0,
        //     receiver: Message::TargetAddress::User(0),
        //     state: Message::State::CreateNewUser("Alice".to_string())
        // };
        // let mut msg = serde_json::to_string(&rq).unwrap();
        // msg.push_str("\n");
        // revc.write_all(msg.as_bytes()).unwrap();
        // println!("Send!");
        let mut ret = String::new();
        // BufReader::new(&revc).read_line(&mut ret).unwrap();
        // println!("Get return msg: {}", ret);
        Self {
            ret,
            view: View::Login,
            uxd: UXD::default(),
            login_view: LoginView::new(UXD::default()),
            popup_view: BasePopup::new(UXD::default()),
            main_view: MainView::new(UXD::default())
        }
    }
}

impl RIMessage {
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
            RMessage::UserCard(m) => self.main_view.msg_user_card(m)
        }
    }
    pub fn subscription(&self) -> iced::Subscription<RMessage> {
        iced::time::every(Duration::from_secs(1)).map(|_| RMessage::Sync)
    }
}