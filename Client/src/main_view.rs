use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use iced::{widget, Task};
use Message::{ChatMessage, LoginReqState, LoginRequest, NPushStream, NReq, NReturnReq, ReturnNLoginReq, ReturnUser, UserInfo, UserRef, is_friend, send, send_receive};
use crate::main_chat_with_page::{ChatWith, MMainChat};
use crate::main_req_page::ReqFriendPage;
use crate::main_search_page::{MMainSearchPage, SearchPage};
use crate::main_view::MMainView::GoPage;
use crate::popup::{Btn, MPopupView};
use crate::state::{RMessage, UXD};
use crate::tab::{MTabView, TabItem, TabView};
use crate::user_card::{MUserCard};


#[derive(Clone, Debug)]
pub enum Page {
    Chat,
    ChatWith(UserRef),
    Search,
    Request
}

#[derive(Clone, Debug)]
pub enum MMainView {
    ReconnectSendStream,
    GoPage(Page),
    MSearchPage(MMainSearchPage),
    SearchUser(u64),
    Send(u64, ChatMessage),
    MainChat(MMainChat)
}

#[derive(Debug)]
pub struct MainView {
    pub req_stream: Option<TcpStream>,
    pub push_stream: Option<TcpStream>,
    pub push_reader: Option<BufReader<TcpStream>>,
    pub user: Option<UserInfo>,
    pub token: String,
    pub uxd: UXD,
    tab_view: TabView,
    page: Page,
    search_page: SearchPage,
    req_page: ReqFriendPage,
    chat_with_page: ChatWith,
    recv: Option<Receiver<NPushStream>>,
    user_name: String
}


impl MainView {
    pub fn new(uxd: UXD) -> Self {
        let tab = TabView::new(vec![
            TabItem {
                name: "Chat".to_string(),
                message: RMessage::MainView(MMainView::GoPage(Page::Chat))
            },
            TabItem {
                name: "Search".to_string(),
                message: RMessage::MainView(MMainView::GoPage(Page::Search))
            },
            TabItem {
                name: "Request".to_string(),
                message: RMessage::MainView(MMainView::GoPage(Page::Request))
            }
        ], uxd.clone());
        let sp = SearchPage::new(UserInfo::default(), uxd.clone());
        let rp = ReqFriendPage::new(uxd.clone());
        let cp = ChatWith::new(uxd.clone());
        Self {
            req_stream: None,
            push_stream: None,
            push_reader: None,
            user: None,
            token: String::new(),
            uxd,
            tab_view: tab,
            page: Page::Chat,
            search_page: sp,
            req_page: rp,
            chat_with_page: cp,
            recv: None,
            user_name: String::new()
        }
    }
    fn chat_page(&self) -> iced::Element<'_, RMessage> {
        if let Some(u) = &self.user {
            let mut names = vec![];
            for i in &u.get_friend() {
                names.push(widget::button(widget::text(i.name.clone()).size(self.uxd.title_font_size).wrapping(widget::text::Wrapping::WordOrGlyph)).on_press(RMessage::MainView(MMainView::GoPage(Page::ChatWith(i.clone())))).into());
            }
            widget::grid(names)
                .fluid(200)
                .spacing(self.uxd.content_space)
                .into()
        } else {
            widget::space().into()
        }
    }
    pub fn view(&self) -> iced::Element<'_, RMessage> {        

        widget::container(
            widget::column![
                widget::row![
                    self.tab_view.view(),
                    widget::text(&self.user_name)
                        .width(iced::Fill)
                        .height(iced::Fill)
                        .align_x(iced::Right)
                        .align_y(iced::Center)
                        .size(self.uxd.title_font_size)
                    ].width(iced::Fill).height(iced::Shrink),
                match &self.page {
                    Page::Chat => self.chat_page(),
                    Page::Search => self.search_page.view(),
                    Page::Request => self.req_page.view(),
                    Page::ChatWith(_) => self.chat_with_page.view(),
                }
            ].width(iced::Fill)
        ).width(iced::Fill).into()
    }
    pub fn update(&mut self, msg: MMainView) -> Task<RMessage> {
        match msg {
            MMainView::ReconnectSendStream => self.try_connect_push_stream(),
            MMainView::GoPage(p) => {
                if let Page::ChatWith(u) = p.clone() {
                    self.chat_with_page.change_user(u);
                }
                self.page = p;
                Task::none()
            },
            MMainView::MSearchPage(m) => self.search_page.update(m),
            MMainView::SearchUser(u) => {
                if let Some(x) = &mut self.req_stream {
                    let ret = send_receive(x, NReq::GetUserFromUid(u));
                    let ret: NReturnReq = serde_json::from_str(ret.as_str()).unwrap();
                    if let NReturnReq::GetUserFromUid(ux) = ret {
                        if let ReturnUser::Ok(ur) = ux {
                            self.search_page.search_card = vec![ur];
                        }
                    }
                }
                Task::none()
            }
            MMainView::Send(t, m) => {
                if let Some(tcp) = &mut self.req_stream {
                    send(tcp, NReq::Single(Message::NReqSigle::Send(Message::TargetAddress::User(t), m)));
                }
                Task::none()
            }
            MMainView::MainChat(m) => self.chat_with_page.update(m)
        }
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
                self.boost_set();
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
        match msg {
            MUserCard::ChatWith(u) => {
                return Task::done(RMessage::MainView(GoPage(Page::ChatWith(u))))
            }
            MUserCard::AddFriend(u) => {
                if let Some(tcp) = &mut self.req_stream {
                    send(tcp, NReq::Single(
                        Message::NReqSigle::AddFriendReq(u))
                    );
                }
            }
            MUserCard::SureFriend(u) => {
                if let Some(rs) = &mut self.req_stream {
                    if let Some(user_info) = &mut self.user {
                        user_info.add_friend(u.clone());
                        let idx = self.req_page.wait_list.iter().position(|x| *x == u);
                        if let Some(i) = idx {
                            self.req_page.wait_list.remove(i);
                        }
                        // println!("Add Fr - {u}");
                        send(rs, NReq::Single(Message::NReqSigle::SureFriendReq(u.uid)));
                        // user_info.friend.push(u.clone());
                    }
                }
            }
            MUserCard::None(u) => {
                let idx = self.req_page.wait_list.iter().position(|x| *x == u);
                if let Some(i) = idx {
                    self.req_page.wait_list.remove(i);
                }
            }
        }
        Task::none()
    }
    pub fn msg_tab_view(&mut self, msg: MTabView) -> Task<RMessage> {
        self.tab_view.update(msg)
    }
    pub fn boost_set(&mut self) {
        if let Some(x) = &self.user {
            self.search_page.user_info = x.clone();
            self.user_name = x.name.clone();
            self.chat_with_page.yo = UserRef { 
                uid: x.uid, 
                name: x.name.clone() 
            }
        }
        let (sender, recv) = mpsc::channel();
        self.recv = Some(recv);
        // let mut push = self.push_stream.take().unwrap();
        let mut reader = self.push_reader.take().unwrap();
        thread::spawn(move || {
            loop {
                let mut msg = String::new();
                match reader.read_line(&mut msg) {
                    Ok(0) => break,
                    Err(_) => break,
                    _ => ()
                }
                let msg: NPushStream = serde_json::from_str(msg.as_str()).unwrap();
                dbg!(&msg);
                sender.send(msg).unwrap();
            }
        });
    }
    pub fn sync(&mut self) {
        // println!("SYNC");
        if let Some(x) = &self.recv {
            if let Ok(m) = x.try_recv() {
                dbg!(&m);
                match m {
                    NPushStream::AddFriendReq(u) => {
                        if let Some(rs) = &mut self.req_stream {
                            let ret = send_receive(rs, NReq::GetUserFromUid(u));
                            let ret:NReturnReq = serde_json::from_str(ret.as_str()).unwrap();
                            if let NReturnReq::GetUserFromUid(ur) = ret {
                                if let ReturnUser::Ok(retu) = ur {
                                    if !self.req_page.wait_list.contains(&retu) {
                                        self.req_page.wait_list.push(retu);
                                    }
                                    // if let Some(user_info) = &mut self.user {
                                    //     user_info.friend.push(retu);
                                    //     println!("Add Fr - {u}");
                                    //     send(rs, NReq::Single(Message::NReqSigle::SureFriendReq(u)));
                                    // }
                                }
                            }
                        }
                    }
                    NPushStream::SureFriendReq(u) => {
                        if let Some(rs) = &mut self.req_stream {
                            let ret = send_receive(rs, NReq::GetUserFromUid(u));
                            let ret:NReturnReq = serde_json::from_str(ret.as_str()).unwrap();
                            if let NReturnReq::GetUserFromUid(ur) = ret {
                                if let ReturnUser::Ok(retu) = ur {
                                    if let Some(user_info) = &mut self.user {
                                        user_info.add_friend(retu);
                                        println!("Add Fr");
                                        if !is_friend(&user_info, u) {
                                            send(rs, NReq::Single(Message::NReqSigle::SureFriendReq(u)));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    NPushStream::Send(u, m) => {
                        println!("Get message: {}", m);
                        self.chat_with_page.record.push_message(u, m);
                    }
                }
            }
        }
    }
}