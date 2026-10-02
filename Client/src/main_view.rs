use std::fmt::Display;
use std::io::BufReader;
use std::net::TcpStream;
use iced::{widget, Task};
use Message::{LoginReqState, LoginRequest, NReq, NReturnReq, ReturnNLoginReq, UserInfo, send_receive};
use crate::popup::{Btn, MPopupView};
use crate::state::{RMessage, UXD};
use crate::user_card::{MUserCard, user_card};

pub enum ChatState {
    View,
    Chat(u64)
}

pub enum Views {
    Chat(ChatState),
    Search
}

impl Views {
    pub fn iter() -> impl Iterator<Item = Views> {
        [
            Views::Chat(ChatState::View),
            Views::Search
        ].into_iter()
    }
}

impl Display for Views {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Views::Chat(_) => write!(f, "聊天"),
            Views::Search => write!(f, "搜索")
        }
    }
}

#[derive(Clone)]
pub enum MMainView {
    ReconnectSendStream,
    REFALLUSER
}

#[derive(Debug)]
pub struct MainView {
    pub req_stream: Option<TcpStream>,
    pub push_stream: Option<TcpStream>,
    push_reader: Option<BufReader<TcpStream>>,
    pub user: Option<UserInfo>,
    pub token: String,
    pub uxd: UXD,
    pub all_user: Vec<UserInfo>
}

impl MainView {
    pub fn new(uxd: UXD) -> Self {
        Self {
            req_stream: None,
            push_stream: None,
            push_reader: None,
            user: None,
            token: String::new(),
            uxd,
            all_user: vec![]
        }
    }
    pub fn view(&self) -> iced::Element<'_, RMessage> {
        
        let mut z = vec![];
        for i in &self.all_user {
            if let Some(x) = &self.user {
                let x = x.clone();
                let uc =  user_card(i.uid, i.name.clone(), x.friend.contains(&i.uid), &self.uxd);
                z.push(uc)
            }
        }
        widget::container(
            widget::column![
                widget::button("刷新").on_press(RMessage::MainView(MMainView::REFALLUSER)),
                widget::column(z).spacing(self.uxd.content_space)
            ]
        ).into()
    }
    pub fn update(&mut self, msg: MMainView) -> Task<RMessage> {
        match msg {
            MMainView::ReconnectSendStream => self.try_connect_push_stream(),
            MMainView::REFALLUSER => self.get_all_users()
        }
    }
    pub fn get_all_users(&mut self) -> Task<RMessage> {
        if let Some(req) = &mut self.req_stream {
            let ret = send_receive(req, NReq::GetAllUsers);
            let ret: NReturnReq = serde_json::from_str(ret.as_str()).unwrap();
            if let NReturnReq::GetAllUsers(a) = ret {
                self.all_user = a
            }
        }

        Task::none()
    }
    pub fn try_connect_push_stream(&mut self) -> Task<RMessage> {
        let mut push_stream = TcpStream::connect("127.0.0.1:7878").unwrap();
        let msg = LoginRequest {
            uid: self.user.clone().unwrap().uid,
            password: String::new(),
            name: String::new(),
            state: LoginReqState::ConversionSendStream(self.token.clone())
        };
        let ret = send_receive(&mut push_stream, msg);
        let ret: ReturnNLoginReq = serde_json::from_str(ret.as_str()).unwrap();
        match ret {
            ReturnNLoginReq::ConversionSendStreamSuccessful => {
                self.push_reader = Some(BufReader::new(push_stream.try_clone().unwrap()));
                self.push_stream = Some(push_stream);
                Task::done(
                    RMessage::PopupView(
                        MPopupView::ShowTextPopupWithOk(
                            "连接成功!".to_string(),
                            "推送流连接成功!".to_string()
                        )
                    )
                )
            },
            ReturnNLoginReq::ConversionSendStreamFail => {
                Task::done(
                    RMessage::PopupView(MPopupView::ShowTextPopupRed(
                        "连接失败!".to_string(),
                        "推送流无法连接!".to_string(),
                        vec![
                            Btn {
                                name: "重试".to_string(),
                                msg: RMessage::MainView(MMainView::ReconnectSendStream)
                            },
                            Btn {
                                name: "退出".to_string(),
                                msg: RMessage::Exit
                            }
                        ]
                    ))
                )
            },
            _ => Task::none()
        }
    }
    pub fn msg_user_card(&mut self, msg: MUserCard) -> Task<RMessage> {
        Task::none()
    }
}