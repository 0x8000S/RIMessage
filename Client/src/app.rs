use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;
use iced;
use iced::widget;
use iced_aw;

use crate::state::{View, RMessage, UXD};
use crate::login::LoginView;


pub struct RIMessage {
    ret: String,
    view: View,
    uxd: UXD,
    login_view: LoginView
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
            login_view: LoginView::new(UXD::default())
        }
    }
}

impl RIMessage {
    fn view_main(&self) -> iced::Element<'_, RMessage> {
        widget::text(format!("{}", self.ret)).into()
    }
    pub fn view(&self) -> iced::Element<'_, RMessage> {
        widget::container(
            match self.view {
                View::Login => self.login_view.view(),
                View::Main => self.view_main()
            }
        ).into()

    }
    pub fn update(&mut self, msg: RMessage) {
        match msg {
            RMessage::Sync => {
                // let mut s = String::new();
                // BufReader::new(&self.tcp).read_line(&mut s).unwrap();
                // self.ret = s;
            }
            RMessage::LoginView(m) => self.login_view.update(m)
        }
    }
    pub fn subscription(&self) -> iced::Subscription<RMessage> {
        iced::time::every(Duration::from_secs(1)).map(|_| RMessage::Sync)
    }
}