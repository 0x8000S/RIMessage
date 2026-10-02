use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Mutex, Arc};
use std::thread::{self, sleep};
use std::time::Duration;
use uuid::Uuid;
use Message::{receive, NReq, send, NReturnReq, ReturnUser};
use crate::user::UserManagement;

#[derive(Debug)]
pub struct OnlinePool {
    pool: HashMap<u64, ConnectBall>,
    usm: Arc<Mutex<UserManagement>>,
    running: bool,
    ssender: Sender<CIMessage>,
    srevc: Receiver<CIMessage>,
}

impl OnlinePool {
    pub fn new(usm: Arc<Mutex<UserManagement>>) -> Self {
        let (bsender, brecv) = mpsc::channel();
        Self {
            pool: HashMap::new(),
            usm,
            running: true,
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
            recv: srecv
        }));
        let mut cb = ConnectBall {
            instance: ci.clone(),
            send: ssender,
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
                    self.pool.remove(&u);
                    println!("在线池id-{u}已被消毁!");
                }
                _ => ()
            }
        }
    }
}


pub enum CIMessage {
    Close(u64),
}


#[derive(Debug)]
pub struct ConnectBall {
    instance: Arc<Mutex<ConnectInstance>>,
    send: mpsc::Sender<CIMessage>,
    // recv: mpsc::Receiver<CIOPMessage>,
    // system_req: 
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
    recv: mpsc::Receiver<CIMessage>,
    sender: mpsc::Sender<CIMessage>
}

impl ConnectInstance {
    pub fn work(&self) {
        let tcpc = self.req_stream.clone();
        let usmc = self.usm.clone();
        let sender = self.sender.clone();
        let uid = self.uid;
        thread::spawn(move || {
                Self::req_thread(tcpc, usmc);
                println!("请求关闭");
                let _ = sender.send(CIMessage::Close(uid));
            }
        );
    }
    pub fn req_thread(tcp_stream: Arc<Mutex<TcpStream>>, usm: Arc<Mutex<UserManagement>>) {
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