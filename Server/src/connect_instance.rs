use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Mutex, Arc};
use std::thread;
use uuid::Uuid;
use Message::{NPushStream, NReq, NReqSigle, NReturnReq, ReturnUser, TargetAddress, receive, send};
use crate::connect_instance::PushCIMessage::SystemReq;
use crate::user::UserManagement;

#[derive(Debug)]
pub struct OnlinePool {
    pool: HashMap<u64, ConnectBall>,
    usm: Arc<Mutex<UserManagement>>,
    ssender: Sender<CIMessage>,
    srevc: Receiver<CIMessage>,
}

impl OnlinePool {
    pub fn new(usm: Arc<Mutex<UserManagement>>) -> Self {
        let (bsender, brecv) = mpsc::channel();
        Self {
            pool: HashMap::new(),
            usm,
            ssender: bsender,
            srevc: brecv
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
        let mut ci = Arc::new(Mutex::new(ConnectInstance {
            uid: uid,
            push_stream: Arc::new(Mutex::new(push)),
            req_stream: Arc::new(Mutex::new(req)),
            usm: self.usm.clone(),
            sender: self.ssender.clone(),
            recv: Arc::new(Mutex::new(srecv))
        }));
        let mut cb = ConnectBall {
            instance: ci.clone(),
            send: ssender,
            willremove: false
        };
        cb.run();
        self.pool.insert(uid, cb);
        println!("用户{uid},成功升入在线池!");
    }
    pub fn message_stream(&mut self) {
        // println!("RUNNING CLEAR THREAD");
        if let Ok(m) = self.srevc.try_recv() {
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
                            if let Some(i) = self.find_mut_instance(u) {
                                i.send.send(CIMessage::Push(PushCIMessage::Send(s, c))).unwrap();
                            }
                        }
                    }
                    ReqCIMessage::ReqAddFriend(s, t) => {
                        if let Some(i) = self.find_mut_instance(t) {
                            i.send.send(CIMessage::Push(PushCIMessage::ReqAddFriend(s))).unwrap();
                        }
                    }
                }
                _ => ()
            }
        }
    }
}

pub enum PushCISystemReq {
    Close,
    ACK(u64)
}

pub enum PushCIMessage {
    Send(u64, String),
    ReqAddFriend(u64),
    SystemReq(PushCISystemReq)
}

pub enum ReqCIMessage {
    Send(u64, TargetAddress, String),
    ReqAddFriend(u64, u64),
}

pub enum CIMessage {
    Close(u64),
    Req(ReqCIMessage),
    Push(PushCIMessage)
}


#[derive(Debug)]
pub struct ConnectBall {
    instance: Arc<Mutex<ConnectInstance>>,
    send: mpsc::Sender<CIMessage>,
    // recv: mpsc::Receiver<CIOPMessage>,
    // system_req: 
    willremove: bool
}

impl ConnectBall {
    pub fn run(&mut self) {
        let worker = self.instance.clone();
        thread::spawn(move || worker.lock().unwrap().work());
    }
}

#[derive(Debug)]
pub struct ConnectInstance {
    uid: u64,
    push_stream: Arc<Mutex<TcpStream>>,
    req_stream: Arc<Mutex<TcpStream>>,
    usm: Arc<Mutex<UserManagement>>,
    recv: Arc<Mutex<mpsc::Receiver<CIMessage>>>,
    sender: mpsc::Sender<CIMessage>
}

impl ConnectInstance {
    pub fn work(&self) {
        let tcpc = self.req_stream.clone();
        let usmc = self.usm.clone();
        let sender = self.sender.clone();
        let esender = self.sender.clone();
        let psender = self.sender.clone();
        let uid = self.uid;
        let ptcpc = self.push_stream.clone();
        let recvc = self.recv.clone();
        thread::spawn(move || {
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
                    let ptcpc = ptcpc.clone();
                    Self::push_thread(ptcpc, msg);
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
    pub fn push_thread(tcp_stream: Arc<Mutex<TcpStream>>, recv: CIMessage) {
        if let CIMessage::Push(pci) = recv {
            match pci {
                PushCIMessage::Send(su, c) => {
                    send(&mut *tcp_stream.lock().unwrap(), NPushStream::Send(su, c));
                }
                PushCIMessage::ReqAddFriend(u) => send(&mut *tcp_stream.lock().unwrap(), NPushStream::AddFriendReq(u)),
                _ => ()
            }
        }
    }        
    pub fn req_thread(tcp_stream: Arc<Mutex<TcpStream>>, usm: Arc<Mutex<UserManagement>>, sender: Sender<CIMessage>, uid: u64) {
        loop {
            let ret = receive(&*tcp_stream.lock().unwrap());
            if ret == "OVER".to_string() {
                return;
            }
            let ret: NReq = serde_json::from_str(ret.as_str()).unwrap();
            match ret {
                NReq::GetAllUsers => {
                    send(&mut *tcp_stream.lock().unwrap(), NReturnReq::GetAllUsers(usm.lock().unwrap().get_all_users()))
                }
                NReq::GetUserFromUid(u) => {
                    match usm.lock().unwrap().find_user(u) {
                        Some(x) => send(&mut *tcp_stream.lock().unwrap(), NReturnReq::GetUserFromUid(
                            ReturnUser::Ok(x.clone())
                        )),
                        None => send(&mut *tcp_stream.lock().unwrap(), NReturnReq::GetUserFromUid(
                            ReturnUser::Error
                        ))
                    }
                }
                NReq::Single(x) => match x {
                    NReqSigle::Send(u, c) => {
                        sender.send(CIMessage::Req(ReqCIMessage::Send(uid, u, c))).unwrap();
                    }
                    NReqSigle::AddFriendReq(u) => {
                        sender.send(CIMessage::Req(ReqCIMessage::ReqAddFriend(uid, u))).unwrap();
                    }
                }
            };
        }
    }
}

#[derive(Debug)]
pub struct ObserverPool {
    pub observer_pool: HashMap<u64, Option<ObserveConnectionInstance>>
}

impl ObserverPool {
    pub fn new() -> Self {
        Self {
            observer_pool: HashMap::new()
        }
    }
    pub fn add_observer(&mut self, oci: ObserveConnectionInstance, uid: u64) {
        self.observer_pool.insert(uid, Some(oci));
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
        let mut del_vec = vec![];
        for (u, o) in &mut self.observer_pool {
            // dbg!(u, &o);
            if let Some(o) = o {
                println!("Some!");
                if o.observe() {
                    println!("用户{u},升入在线池");
                    online_pool.lock().unwrap().add_instance(o.push_stream.take().unwrap(), o.req_stream.take().unwrap(), *u);
                    del_vec.push(*u);
                }
            }
        }
        for u in del_vec {
            self.observer_pool.remove(&u);
        }
    }
}

#[derive(Debug)]
pub struct ObserveConnectionInstance {
    pub push_stream: Option<TcpStream>,
    pub req_stream: Option<TcpStream>,
    pub token: String
}

impl ObserveConnectionInstance {
    pub fn new() -> Self {
        Self {
            push_stream: None,
            req_stream: None,
            token: Uuid::new_v4().to_string()
        }
    }
    pub fn observe(&mut self) -> bool {
        self.push_stream.is_some() && self.req_stream.is_some()
    }
}