use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender, channel};
use std::thread;
use iced::{widget, Task};
use Message::{ChatMessage, FromAddress, GroupHeader, GroupRef, LoginReqState, LoginRequest, NPushStream, NReq, NReturnReq, ReturnNLoginReq, TargetAddress, UserId, UserInfo, UserRef, send, send_receive};
use crate::main_chat_with_page::{ChatWith, MMainChat};
use crate::main_req_page::{CardMode, CardUnit, ReqFriendPage, ReqFriendPageData};
use crate::main_search_page::{MMainSearchPage, SearchPage};
use crate::main_view::MMainView::GoPage;
use crate::popup::{Btn, MPopupView, base};
use crate::state::{RMessage, UXD};
use crate::tab::{MTabView, TabItem, TabView};
use crate::user_card::{MUserCard, RecordManagement};
use crate::user_select::{MMainUserSelect, UserSelect};


#[derive(Clone, Debug)]
pub enum Page {
    Chat,
    ChatWith(UserRef),
    Search,
    Request,
    Group,
    GroupWith(GroupRef)
}

#[derive(Clone, Debug)]
pub enum MMainView {
    ReconnectSendStream,
    GoPage(Page),
    MSearchPage(MMainSearchPage),
    SearchUser(UserId),
    Send(TargetAddress, ChatMessage),
    MainChat(MMainChat),
    SwitchCreateGroupMsgState(bool),
    CreateGroup,
    WhenGroupNameType(String),
    MUserSelect(MMainUserSelect),
    ShowSelectUserMessageBox,
    WhenSelectUserDone
}

#[derive(Debug)]
struct DataUnion {
    req_wait_list: Arc<Mutex<ReqFriendPageData>>,
    record: Arc<Mutex<RecordManagement>>
}

enum MainScrDown {
    Msg(UserId)
}

impl DataUnion {
    fn new() -> Self {
        Self {
            req_wait_list: Arc::new(Mutex::new(ReqFriendPageData { wait_list: Vec::new() })),
            record: Arc::new(Mutex::new(RecordManagement::new()))
        }
    }
}

#[derive(Debug)]
pub struct MainView {
    pub req_stream: Option<TcpStream>,
    pub push_stream: Option<TcpStream>,
    pub push_reader: Option<BufReader<TcpStream>>,
    pub user: Option<Arc<Mutex<UserInfo>>>,
    pub token: String,
    pub uxd: UXD,
    data_union: Arc<DataUnion>,
    tab_view: TabView,
    page: Page,
    search_page: SearchPage,
    req_page: ReqFriendPage,
    chat_with_page: ChatWith,
    recv: Option<Receiver<NPushStream>>,
    user_name: String,
    chat_msg_recv: Option<Receiver<MainScrDown>>,
    group_name: String,
    show_create_group_msg: bool,
    user_select_popup: UserSelect
}


impl MainView {
    pub fn new(uxd: UXD) -> Self {
        let tab = TabView::new(vec![
            TabItem {
                name: "Chat".to_string(),
                message: RMessage::MainView(MMainView::GoPage(Page::Chat))
            },
            TabItem {
                name: "Group".to_string(),
                message: RMessage::MainView(MMainView::GoPage(Page::Group))
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
            uxd: uxd.clone(),
            tab_view: tab,
            page: Page::Chat,
            search_page: sp,
            req_page: rp,
            chat_with_page: cp,
            recv: None,
            user_name: String::new(),
            data_union: Arc::new(DataUnion::new()),
            chat_msg_recv: None,
            group_name: String::new(),
            show_create_group_msg: false,
            user_select_popup: UserSelect::new(uxd)
        }
    }
    fn create_group_msg(&self) -> iced::Element<'_, RMessage> {
        base(widget::column![
            widget::text("群组名称").size(self.uxd.title_font_size),
            widget::text_input("键入群组名称", &self.group_name).on_input(|x|RMessage::MainView(MMainView::WhenGroupNameType(x))),
            widget::container(
                widget::row![
                    widget::button("取消").on_press(RMessage::MainView(MMainView::SwitchCreateGroupMsgState(false))).style(widget::button::secondary), 
                    widget::button("创建").on_press(RMessage::MainView(MMainView::CreateGroup))
                ].spacing(self.uxd.content_space)
                ).width(iced::Shrink).padding(self.uxd.view_padding).align_x(iced::Right)
        ].spacing(self.uxd.content_space).padding(self.uxd.view_padding), self.uxd.clone())
    }
    fn chat_page(&self) -> iced::Element<'_, RMessage> {
        if let Some(u) = &self.user {
            let mut names = vec![];
            for i in &u.lock().unwrap().get_friend() {
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
    fn group_page(&self) -> iced::Element<'_, RMessage> {
        if let Some(u) = &self.user {
            let mut names = vec![];
            for i in &u.lock().unwrap().get_group() {
                names.push(widget::button(widget::text(i.name.clone()).size(self.uxd.title_font_size).wrapping(widget::text::Wrapping::WordOrGlyph)).on_press(RMessage::MainView(MMainView::GoPage(Page::GroupWith(i.clone())))).into());
            }
            widget::column![
                widget::button("创建群组").on_press(RMessage::MainView(MMainView::SwitchCreateGroupMsgState(true))),
                widget::grid(names)
                    .fluid(200)
                    .spacing(self.uxd.content_space)
            ].spacing(self.uxd.content_space).width(iced::Fill).height(iced::Fill).into()
        } else {
            widget::space().into()
        }
    }
    pub fn base_view(&self) -> iced::Element<'_, RMessage> {
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
                    Page::Request => self.req_page.view(&self.data_union.clone().req_wait_list.lock().unwrap().wait_list),
                    Page::ChatWith(_) => self.chat_with_page.chat_view(&self.data_union.clone().record.lock().unwrap()),
                    Page::Group => self.group_page(),
                    Page::GroupWith(_) => self.chat_with_page.group_view(&self.data_union.clone().record.lock().unwrap())
                }
            ].width(iced::Fill)
        ).width(iced::Fill).height(iced::Fill).into()
    }
    pub fn view(&self) -> iced::Element<'_, RMessage> {      
        widget::stack![
            self.base_view(),
            self.show_create_group_msg.then(|| self.create_group_msg()).unwrap_or_else(|| widget::space().width(iced::Fill).height(iced::Fill).into()),
            self.user_select_popup.view()
        ].width(iced::Fill).height(iced::Fill).into()
    }
    pub fn update(&mut self, msg: MMainView) -> Task<RMessage> {
        match msg {
            MMainView::ReconnectSendStream => self.try_connect_push_stream(),
            MMainView::GoPage(p) => {
                self.page = p.clone();
                if let Page::ChatWith(u) = p {
                    self.chat_with_page.change_user(u);
                    return self.chat_with_page.chat_slide_down()
                }
                if let Page::GroupWith(g) = p {
                    if let Some(tcp) = &mut self.req_stream {
                        let ret = send_receive(tcp, NReq::GetGroupFromId(g.id));
                        let ret: NReturnReq = serde_json::from_str(ret.as_str()).unwrap();
                        if let NReturnReq::GetGroupFromId(gh) = ret {
                            if gh.is_some() {
                                self.chat_with_page.change_group(gh.unwrap());
                            }
                        }
                    }
                }
                Task::none()
            },
            MMainView::MSearchPage(m) => self.search_page.update(m),
            MMainView::SearchUser(u) => {
                if let Some(x) = &mut self.req_stream {
                    let ret = send_receive(x, NReq::GetUserFromUid(u));
                    let ret: NReturnReq = serde_json::from_str(ret.as_str()).unwrap();
                    if let NReturnReq::GetUserFromUid(ux) = ret {
                        if let Some(ur) = ux {
                            self.search_page.search_card = vec![ur];
                        }
                    }
                }
                Task::none()
            }
            MMainView::Send(t, m) => {
                match t {
                    TargetAddress::User(u) => {
                        self.data_union.record.lock().unwrap().push_message(u, m.clone());
                        if let Some(tcp) = &mut self.req_stream {
                            send(tcp, NReq::Single(Message::NReqSigle::Send(Message::TargetAddress::User(u), m)));
                            return self.chat_with_page.chat_slide_down();
                        }
                    }
                    TargetAddress::Group(g) => {
                        if let Some(tcp) = &mut self.req_stream {
                            send(tcp, NReq::Single(Message::NReqSigle::Send(Message::TargetAddress::Group(g), m)));
                        }
                    }
                }
                Task::none()
            }
            MMainView::MainChat(m) => self.chat_with_page.update(m),
            MMainView::CreateGroup => {
                if let Some(tcp) = &mut self.req_stream {
                    let ret = send_receive(tcp, NReq::CreateGroup(self.group_name.clone()));
                    let ret: NReturnReq = serde_json::from_str(ret.as_str()).unwrap();
                    if let NReturnReq::CreateGroup(x) = ret {
                        if let Some(u) = &self.user {
                            u.lock().unwrap().add_group(x);
                        }
                    }
                    return Task::done(RMessage::MainView(MMainView::SwitchCreateGroupMsgState(false)))
                }
                Task::none()
            }
            MMainView::SwitchCreateGroupMsgState(s) => {
                self.show_create_group_msg = s;
                s.then(||()).unwrap_or_else(|| self.group_name.clear());
                Task::none()
            }
            MMainView::WhenGroupNameType(s) => {
                self.group_name = s;
                Task::none()
            }
            MMainView::MUserSelect(m) => self.user_select_popup.update(m),
            MMainView::ShowSelectUserMessageBox => {
                if let Some(user) = &self.user {
                    self.user_select_popup.user = user.lock().unwrap().clone();
                }
                self.user_select_popup.show = true;
                self.user_select_popup.set_callback(RMessage::MainView(MMainView::WhenSelectUserDone));
                Task::none()
            }
            MMainView::WhenSelectUserDone => {
                if let Some(tcp) = &mut self.req_stream {
                    send(tcp, NReq::Single(Message::NReqSigle::InviteGroupReq(self.user_select_popup.select, self.chat_with_page.talk_group.lock().unwrap().id)));
                }
                Task::none()
            }
        }
    }
    pub fn try_connect_push_stream(&mut self) -> Task<RMessage> {
        let mut push_stream = TcpStream::connect("127.0.0.1:7878").unwrap();
        let msg = LoginRequest {
            uid: self.user.clone().unwrap().lock().unwrap().uid,
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
                    if let Some(user_info) = &self.user {
                        user_info.lock().unwrap().add_friend(u.clone());
                        self.data_union.req_wait_list.lock().unwrap().remove(u.clone().uid, CardMode::ReqFriend);
                        send(rs, NReq::Single(Message::NReqSigle::SureFriendReq(u.uid)));
                    }
                }
            }
            MUserCard::ReqFriendNone(u) => self.data_union.req_wait_list.lock().unwrap().remove(u.uid, CardMode::ReqFriend),
            MUserCard::ReqIntoGroupNone(u) => self.data_union.req_wait_list.lock().unwrap().remove(u.uid, CardMode::ReqIntoGroup),
            MUserCard::SureGroup(gr) => {
                if let Some(u) = &self.user {
                    u.lock().unwrap().add_group(gr.clone());
                    self.data_union.req_wait_list.lock().unwrap().remove(gr.clone().id, CardMode::ReqIntoGroup);
                    if let Some(tcp) = &mut self.req_stream {
                        send(tcp, NReq::Single(Message::NReqSigle::SureGroupReq(gr.id)));
                    }
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
            self.user_name = x.lock().unwrap().name.clone();
            self.chat_with_page.take_user(x.clone());
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
        if let Some(tcp) = &mut self.req_stream {
            let du = self.data_union.clone();
            let u = self.user.clone().unwrap().clone();
            let tcp = tcp.try_clone().unwrap();
            let recv = self.recv.take().unwrap();
            let (st, rv) = channel();
            let talk_group = self.chat_with_page.talk_group.clone();
            self.chat_msg_recv = Some(rv);
            thread::spawn(move || {
                Self::sync(tcp.try_clone().unwrap(), recv, du, u, st, talk_group);
            });
        }
    }
    pub fn tick(&mut self) -> Task<RMessage> {
        if let Some(recv) = &self.chat_msg_recv {
            if let Ok(m) = recv.try_recv() {
                let MainScrDown::Msg(u) = m;
                if let Page::ChatWith(nu) = &self.page {
                    if u == nu.uid {
                        return self.chat_with_page.chat_slide_down()
                    }
                }
            }
        }
        Task::none()
    }
    pub fn get_user_ref(req: &mut TcpStream, uid: UserId) -> Option<UserRef> {
        let ret = send_receive(req, NReq::GetUserFromUid(uid));
        let ret:NReturnReq = serde_json::from_str(ret.as_str()).unwrap();
        if let NReturnReq::GetUserFromUid(ur) = ret {
            if let Some(retu) = ur {
                return Some(retu);
            }
        }
        None
    }
    fn sync(mut req: TcpStream, recv: Receiver<NPushStream>, data_union: Arc<DataUnion>, user: Arc<Mutex<UserInfo>>, send: Sender<MainScrDown>, talk_group: Arc<Mutex<GroupHeader>>) {
        loop {
            if let Ok(m) = recv.recv() {
                dbg!(&m);
                match m {
                    NPushStream::AddFriendReq(u) => {
                        if let Some(ur) = Self::get_user_ref(&mut req, u) {
                            if data_union.req_wait_list.lock().unwrap().wait_list.iter().find(|x| x.mode == CardMode::ReqFriend && x.user.uid == u).is_none() {
                                data_union.req_wait_list.lock().unwrap().wait_list.push(CardUnit {
                                    user: ur,
                                    mode: CardMode::ReqFriend,
                                    gr: None
                                });
                            }
                        }
                    }
                    NPushStream::SureFriendReq(u) => {
                        if let Some(ur) = Self::get_user_ref(&mut req, u) {
                            user.lock().unwrap().add_friend(ur);
                            println!("Add Fr");
                        }
                    }
                    NPushStream::Send(u, m) => {
                        println!("Get message: {}", m);
                        match u {
                            FromAddress::User(mu) => {
                                data_union.record.lock().unwrap().push_message(mu, m);
                                send.send(MainScrDown::Msg(mu)).unwrap();
                            }
                            FromAddress::Group(gi, _mu) => {
                                data_union.record.lock().unwrap().push_group_message(gi, m);
                                
                                // Wait for write.
                            }
                        }
                    }
                    NPushStream::InviteGroupReq(u, gr) => {
                        if let Some(ur) = Self::get_user_ref(&mut req, u) {
                            if data_union.req_wait_list.lock().unwrap().wait_list.iter().find(|x| x.mode == CardMode::ReqIntoGroup && x.user.uid == u).is_none() {
                                data_union.req_wait_list.lock().unwrap().wait_list.push(CardUnit {
                                    user: ur,
                                    mode: CardMode::ReqIntoGroup,
                                    gr: Some(gr)
                                });
                            }
                        }
                    }
                    NPushStream::SureGroupReq(s, g) => {
                        let ret = send_receive(&mut req, NReq::GetGroupFromId(g));
                        let ret: NReturnReq = serde_json::from_str(ret.as_str()).unwrap();
                        if let NReturnReq::GetGroupFromId(x) = ret {
                            if x.is_some() {
                                if user.lock().unwrap().uid == s {
                                    user.lock().unwrap().add_group(x.clone().unwrap().to_ref());
                                }
                                if talk_group.lock().unwrap().id == g {
                                    *talk_group.lock().unwrap() = x.unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}