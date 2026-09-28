use std::io::{Write, BufReader, BufRead};
use std::net::TcpStream;
use iced::{widget, Theme};
use Message::{send_receive, LoginReqState, LoginRequest, ReturnNLoginReq};
use crate::state::{RMessage, UXD};

#[derive(Clone)]
pub enum MLoginView {
    WhenUidType(String),
    WhenPassWordType(String),
    OnSubmitClicked,
    OnCLSClicked
}

#[derive(Eq, PartialEq, Debug)]
enum ViewState {
    Login,
    SignUp
}

#[derive(Debug)]
pub struct LoginView {
    uid: String,
    password: String,
    token: i32,
    login_stream: Option<TcpStream>,
    uxd: UXD,
    tip: String,
    state: ViewState
}

impl LoginView {
    pub fn new(uxd: UXD) -> Self {
        let tcp = TcpStream::connect("127.0.0.1:7878").unwrap();
        Self {
            uid: String::new(),
            password: String::new(),
            token: -1,
            login_stream: Some(tcp),
            uxd,
            tip: String::new(),
            state: ViewState::Login
        }
    }
    pub fn view(&self) -> iced::Element<'_, RMessage> {
        widget::container(
            widget::container(
                widget::column![
                        widget::text((self.state == ViewState::Login).then(|| "登录").unwrap_or_else(|| "注册")).size(48),
                        widget::text_input(
                            (self.state == ViewState::Login).then(|| "键入UID").unwrap_or_else(|| "键入昵称"),
                            &self.uid).width(400).on_input(|x| RMessage::LoginView(MLoginView::WhenUidType(x))),
                        widget::text_input("键入密码", &self.password).width(400).on_input(|x| RMessage::LoginView(MLoginView::WhenPassWordType(x))),
                        widget::button((self.state == ViewState::Login).then(|| "注册账号").unwrap_or_else(|| "登录账号")).style(|t: &Theme, _s| widget::button::Style {
                            background: None,
                            text_color: t.palette().primary,
                            border: Default::default(),
                            shadow: Default::default(),
                            snap: false}).on_press(RMessage::LoginView(MLoginView::OnCLSClicked)),
                        widget::button((self.state == ViewState::Login).then(|| "登录").unwrap_or_else(|| "注册")).on_press(RMessage::LoginView(MLoginView::OnSubmitClicked)),
                        widget::text(&self.tip).style(widget::text::danger)
                    ].align_x(iced::Center).width(iced::Shrink).spacing(self.uxd.content_space)
            )
                .width(iced::Fill)
                .height(iced::Fill)
                .align_x(iced::Center)
                .align_y(iced::Center)
        )
            .width(iced::Fill)
            .height(iced::Fill)
            .into()
    }
    pub fn update(&mut self, msg: MLoginView) {
        match msg {
            MLoginView::WhenUidType(u) => {
                if self.state == ViewState::Login {
                    if let Ok(v) = u.parse::<u64>() {
                        self.uid = u;
                    }
                } else {
                    self.uid = u;
                }
            }
            MLoginView::WhenPassWordType(p) => self.password = p,
            MLoginView::OnSubmitClicked => {
                match &self.state {
                    ViewState::Login => self.login(),
                    ViewState::SignUp => self.signup()
                }
            }
            MLoginView::OnCLSClicked => {
                self.uid.clear();
                self.password.clear();
                match &self.state {
                    ViewState::Login => self.state = ViewState::SignUp,
                    ViewState::SignUp => self.state = ViewState::Login
                }
            }
        }
    }
    pub fn login(&mut self) {
        dbg!(&self);
        let rl = LoginRequest {
            uid: self.uid.parse().unwrap(),
            password: self.password.clone(),
            name: String::new(),
            state: LoginReqState::Login
        };
        if let Some(t) = &mut self.login_stream {
            println!("{:?}", rl);
            // t.write_all(m.as_bytes()).unwrap();
            // println!("Send login msg!");
            let ret = send_receive(t, rl);
            let ret: ReturnNLoginReq = serde_json::from_str(ret.as_str()).unwrap();
            match ret {
                ReturnNLoginReq::OK(u, t) => println!("成功登陆,欢迎{}\n{:?}\n你的临时认证token为: {t}", u.name, u),
                ReturnNLoginReq::PasswordError => self.tip = String::from("密码错误"),
                ReturnNLoginReq::UserNotFound => self.tip = String::from("无效用户"),
                ReturnNLoginReq::UserCreateOK(u) => println!("用户创建成功, uid: {u}")
            }
        }
    }
    pub fn signup(&mut self) {
        dbg!(&self);
        let rl = LoginRequest {
            uid: 0,
            password: self.password.clone(),
            name: self.uid.clone(),
            state: LoginReqState::SignUp
        };
        if let Some(t) = &mut self.login_stream {
            let ret = send_receive(t, rl);
            let ret: ReturnNLoginReq = serde_json::from_str(ret.as_str()).unwrap();
            if let ReturnNLoginReq::UserCreateOK(u) = ret {
                println!("目标用户创建成功!\nuid: {u}");
            }
        }
    }
}
