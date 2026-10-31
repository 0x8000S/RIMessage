use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Mutex, Arc};
use std::thread;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use Message::{ChatMessage, GroupId, GroupRef, NPushStream, NReq, NReqSigle, NReturnReq, ReturnUser, TargetAddress, UserId, receive, send};
use crate::connect_instance::PushCIMessage::SystemReq;
use crate::user::UserManagement;

#[derive(Debug)]
pub struct OfflineMessagePool {
    pool: HashMap<u64, Vec<CIMessage>>
}

impl OfflineMessagePool {
    pub fn new() -> Self {
        Self { pool: HashMap::new() }
    }
    pub fn add_message(&mut self, uid: u64, msg: CIMessage) {
        if let Some(x) = self.pool.get_mut(&uid) {
            x.push(msg);
            return;
        }
        self.pool.insert(uid, vec![msg]);
    }
    pub fn get_message(&mut self, uid: u64) -> Option<Vec<CIMessage>> {
        if let Some(x) = self.pool.remove(&uid) {
            return Some(x)
        }
        None
    }
}

#[derive(Debug)]
pub struct OnlinePool {
    pool: HashMap<u64, ConnectBall>,
    usm: Arc<Mutex<UserManagement>>,
    ssender: Sender<CIMessage>,
    srevc: Receiver<CIMessage>,
    offline_pool: OfflineMessagePool
}

impl OnlinePool {
    pub fn new(usm: Arc<Mutex<UserManagement>>) -> Self {
        let (bsender, brecv) = mpsc::channel();
        Self {
            pool: HashMap::new(),
            usm,
            ssender: bsender,
            srevc: brecv,
            offline_pool: OfflineMessagePool::new()
        }
    }
    pub fn find_instance(&self, uid: u64) -> Option<&ConnectBall> {
        self.pool.get(&uid)
    }
    pub fn find_mut_instance(&mut self, uid: u64) -> Option<&mut ConnectBall> {
        self.pool.get_mut(&uid)
    }
    pub fn add_instance(&mut self, push: TcpStream, req: TcpStream, uid: u64) {
        let (ssender, srecv) = mpsc::channel();
        let ci = Arc::new(Mutex::new(ConnectInstance {
            uid: uid,
            push_stream: Some(push),
            req_stream: Some(req),
            usm: self.usm.clone(),
            sender: self.ssender.clone(),
            recv: Arc::new(Mutex::new(srecv))
        }));
        let mut cb = ConnectBall {
            instance: ci.clone(),
            send: ssender,
            willremove: false
        };
        if let Some(m) = self.offline_pool.get_message(uid) {
            cb.run(m);
        } else {
            cb.run(vec![]);
        }
        self.pool.insert(uid, cb);
        println!("用户{uid},成功升入在线池!");
    }
    fn send_push_msg(&mut self, uid: u64, msg: CIMessage) {
        if let Some(i) = self.find_mut_instance(uid) {
            i.send.send(msg).unwrap();
        } else {
            println!("消息: {msg:?} 进入离线池");
            self.offline_pool.add_message(uid, msg);
        }
    }
    pub fn message_stream(&mut self) {
        while let Ok(m) = self.srevc.try_recv() {
            match m {
                CIMessage::Close(u) => {
                    if let Some(i) = self.find_mut_instance(u) {
                        i.send.send(CIMessage::Push(SystemReq(PushCISystemReq::Close))).unwrap();
                        i.willremove = true;
                    }
                }
                CIMessage::Push(x) => match x {
                    PushCIMessage::SystemReq(x) => match x {
                        PushCISystemReq::ACK(u) => {
                            if let Some(i) = self.find_mut_instance(u) {
                                if i.willremove {
                                    self.pool.remove(&u);
                                    println!("在线池id-{u}已被消毁!");
                                }
                            }
                        }
                        _ => ()
                    }
                    _ => ()
                }
                CIMessage::Req(x) => match x {
                    ReqCIMessage::Send(s, t, c) => {
                        if let TargetAddress::User(u) = t {
                            self.send_push_msg(u, CIMessage::Push(PushCIMessage::Send(s, c)));
                        }
                    }
                    ReqCIMessage::ReqAddFriend(s, t) => {
                        self.send_push_msg(t, CIMessage::Push(PushCIMessage::ReqAddFriend(s)));
                    }
                    ReqCIMessage::ReqSureFriend(s, t) => {
                        self.send_push_msg(t, CIMessage::Push(PushCIMessage::ReqSureFriend(s)));
                    }
                    ReqCIMessage::ReqInviteGroup(s, t, gr) => {
                        self.send_push_msg(t, CIMessage::Push(PushCIMessage::ReqInviteGroup(s, gr)));
                    }
                    ReqCIMessage::ReqSureGroup(s, group) => {
                        let users = self.usm.lock().unwrap().get_group_member(group).unwrap_or_else(|| vec![]);
                        for u in users {
                            self.send_push_msg(u, CIMessage::Push(PushCIMessage::ReqSureGroup(s, group)));
                        }
                    }
                }
            }
        }
    }
    pub fn get_data_ref(&self) -> Vec<ConnectBallRef> {
        let mut ret = vec![];
        for (u, _) in &self.pool {
            ret.push(ConnectBallRef {
                uid: *u
            });
        }
        ret
    }
}

#[derive(Debug, Clone)]
pub enum PushCISystemReq {
    Close,
    ACK(u64)
}

#[derive(Debug, Clone)]
pub enum PushCIMessage {
    Send(u64, ChatMessage),
    ReqAddFriend(u64),
    ReqSureFriend(u64),
    ReqInviteGroup(u64, GroupRef),
    SystemReq(PushCISystemReq),
    ReqSureGroup(UserId, GroupId)
}

#[derive(Debug, Clone)]
pub enum ReqCIMessage {
    Send(UserId, TargetAddress, ChatMessage),
    ReqAddFriend(UserId, UserId),
    ReqSureFriend(UserId, UserId),
    ReqInviteGroup(UserId, UserId, GroupRef),
    ReqSureGroup(UserId, GroupId)
}

#[derive(Debug, Clone)]
pub enum CIMessage {
    Close(UserId),
    Req(ReqCIMessage),
    Push(PushCIMessage)
}

#[derive(Deserialize, Serialize)]
pub struct ConnectBallRef {
    pub uid: UserId
}

#[derive(Debug)]
pub struct ConnectBall {
    instance: Arc<Mutex<ConnectInstance>>,
    send: mpsc::Sender<CIMessage>,
    willremove: bool
}

impl ConnectBall {
    pub fn run(&mut self, offline_msg: Vec<CIMessage>) {
        let worker = self.instance.clone();
        thread::spawn(move || worker.lock().unwrap().work());
        for i in offline_msg {
            let _ = self.send.send(i);
        }
    }
}

#[derive(Debug)]
pub struct ConnectInstance {
    uid: u64,
    push_stream: Option<TcpStream>,
    req_stream: Option<TcpStream>,
    usm: Arc<Mutex<UserManagement>>,
    recv: Arc<Mutex<mpsc::Receiver<CIMessage>>>,
    sender: mpsc::Sender<CIMessage>
}

impl ConnectInstance {
    pub fn work(&mut self) {
        let tcpc = self.req_stream.take().unwrap();
        let usmc = self.usm.clone();
        let sender = self.sender.clone();
        let esender = self.sender.clone();
        let psender = self.sender.clone();
        let uid = self.uid;
        let ptcpc = self.push_stream.take().unwrap();
        let recvc = self.recv.clone();
        thread::spawn(move || {
            let mut ptcpc = ptcpc;
            loop {
                while let Ok(msg) = recvc.lock().unwrap().recv() {
                    if let CIMessage::Push(x) = &msg {
                        if let PushCIMessage::SystemReq(x) = x {
                            match x {
                                PushCISystemReq::Close => {
                                    println!("System-退出");
                                    psender.send(CIMessage::Push(PushCIMessage::SystemReq(PushCISystemReq::ACK(uid)))).unwrap();
                                    return;
                                }
                                _ => ()
                            }
                        }
                    }
                    Self::push_thread(&mut ptcpc, msg);
                }
            }
        });
        thread::spawn(move || {
                Self::req_thread(tcpc, usmc, sender, uid);
                println!("请求关闭");
                let _ = esender.send(CIMessage::Close(uid));
            }
        );
    }
    pub fn push_thread(tcp_stream: &mut TcpStream, recv: CIMessage) {
        if let CIMessage::Push(pci) = recv {
            match pci {
                PushCIMessage::Send(su, c) => {
                    send(tcp_stream, NPushStream::Send(su, c));
                }
                PushCIMessage::ReqAddFriend(u) => send(tcp_stream, NPushStream::AddFriendReq(u)),
                PushCIMessage::ReqSureFriend(u) => send(tcp_stream, NPushStream::SureFriendReq(u)),
                PushCIMessage::ReqInviteGroup(s, t) => send(tcp_stream, NPushStream::InviteGroupReq(s, t)),
                PushCIMessage::ReqSureGroup(u, g) => send(tcp_stream, NPushStream::SureGroupReq(u, g)),
                _ => ()
            }
        }
    }        
    pub fn req_thread(mut tcp_stream: TcpStream, usm: Arc<Mutex<UserManagement>>, sender: Sender<CIMessage>, uid: u64) {
        loop {
            let ret = receive(&tcp_stream);
            if ret == "OVER".to_string() {
                return;
            }
            let ret: NReq = serde_json::from_str(ret.as_str()).unwrap();
            match ret {
                NReq::GetAllUsers => {
                    send(&mut tcp_stream, NReturnReq::GetAllUsers(usm.lock().unwrap().get_all_users()))
                }
                NReq::GetUserFromUid(u) => {
                    match usm.lock().unwrap().find_user_ref(u) {
                        Some(x) => send(&mut tcp_stream, NReturnReq::GetUserFromUid(
                            ReturnUser::Ok(x.clone())
                        )),
                        None => send(&mut tcp_stream, NReturnReq::GetUserFromUid(
                            ReturnUser::Error
                        ))
                    }
                }
                NReq::CreateGroup(n) => {
                    println!("User-{uid} created the group");
                    let idx = usm.lock().unwrap().create_group(uid, n);
                    send(&mut tcp_stream, NReturnReq::CreateGroup(idx));
                }
                NReq::Single(x) => match x {
                    NReqSigle::Send(u, c) => {
                        sender.send(CIMessage::Req(ReqCIMessage::Send(uid, u, c))).unwrap();
                    }
                    NReqSigle::AddFriendReq(u) => {
                        sender.send(CIMessage::Req(ReqCIMessage::ReqAddFriend(uid, u))).unwrap();
                    },
                    NReqSigle::SureFriendReq(u) => {
                        println!("SureFd from {uid} to {u}");
                        usm.lock().unwrap().change_friend(uid, u, true);
                        sender.send(CIMessage::Req(ReqCIMessage::ReqSureFriend(uid, u))).unwrap();
                    }
                    NReqSigle::InviteGroupReq(t, gi) => {
                        if let Some(gr) = usm.lock().unwrap().get_group_ref(gi) {
                            sender.send(CIMessage::Req(ReqCIMessage::ReqInviteGroup(uid, t, gr))).unwrap();
                        }
                    }
                    NReqSigle::SureGroupReq(g) => {
                        usm.lock().unwrap().add_unit(g, uid);
                        sender.send(CIMessage::Req(ReqCIMessage::ReqSureGroup(uid, g))).unwrap();
                    }
                }
            };
        }
    }
}

pub enum MOP {
    Boost(u64)
}

#[derive(Debug)]
pub struct ObserverPool {
    pub observer_pool: HashMap<u64, Option<ObserveConnectionInstance>>,
    _sender: Sender<MOP>,
    recv: Receiver<MOP>
}

impl ObserverPool {
    pub fn new() -> Self {
        let (sd, rv) = mpsc::channel();
        Self {
            observer_pool: HashMap::new(),
            _sender: sd,
            recv: rv
        }
    }
    pub fn add_observer(&mut self, uid: u64) -> String {
        let oci = ObserveConnectionInstance::new(self._sender.clone(), uid);
        let token = oci.token.clone();
        self.observer_pool.insert(uid, Some(oci));
        token
    }
    pub fn find_observer(&mut self, uid: u64) -> Option<&mut ObserveConnectionInstance> {
        let ret = self.observer_pool.get_mut(&uid).unwrap();
        match ret {
            Some(v) => Some(v),
            None => None
        }
    }
    pub fn watch(&mut self, online_pool: Arc<Mutex<OnlinePool>>) {
        // dbg!(&self);
        while let Ok(m) = self.recv.try_recv() {
            let MOP::Boost(u) = m;
            println!("用户{u},升入在线池");
            let ins = self.find_observer(u).unwrap();
            online_pool.lock().unwrap().add_instance(ins.push_stream.take().unwrap(), ins.req_stream.take().unwrap(), u);
            self.observer_pool.remove(&u);
            
        }
    }
    pub fn get_data_ref(&self) -> Vec<ObserveConnectionInstanceRef> {
        let mut ret = vec![];
        for (u, o) in &self.observer_pool {
            if let Some(x) = o {
                ret.push(ObserveConnectionInstanceRef {
                    token: x.token.clone(),
                    uid: *u
                });
            }
        }
        ret
    }
}

#[derive(Deserialize, Serialize)]
pub struct ObserveConnectionInstanceRef {
    pub token: String,
    pub uid: u64
}

#[derive(Debug)]
pub struct ObserveConnectionInstance {
    push_stream: Option<TcpStream>,
    req_stream: Option<TcpStream>,
    pub token: String,
    send: Sender<MOP>,
    uid: u64
}

impl ObserveConnectionInstance {
    pub fn new(send: Sender<MOP>, uid: u64) -> Self {
        Self {
            push_stream: None,
            req_stream: None,
            token: Uuid::new_v4().to_string(),
            send: send,
            uid
        }
    }
    pub fn observe(&mut self) {
        if self.push_stream.is_some() && self.req_stream.is_some() {
            println!("UID={} 观察池已成功匹配", self.uid);
            self.send.send(MOP::Boost(self.uid)).unwrap();
        }
    }
    pub fn add_push(&mut self, tcp: TcpStream) {
        self.push_stream = Some(tcp);
        self.observe();
    }
    pub fn add_req(&mut self, tcp: TcpStream) {
        self.req_stream = Some(tcp);
        self.observe();
    }
}