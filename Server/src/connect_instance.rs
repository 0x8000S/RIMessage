use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{mpsc, Mutex, Arc};
use uuid::Uuid;
use crate::user::UserManagement;

pub struct  OnlinePool {
    pool: HashMap<u64, ConnectInstance>
}

impl OnlinePool {
    pub fn new() -> Self {
        Self {
            pool: HashMap::new()
        }
    }
    pub fn find_instance(&self, uid: u64) -> Option<&ConnectInstance> {
        self.pool.get(&uid)
    }
}

pub struct CIReturn {

}

pub struct CIRequest {

}

pub struct ConnectInstance {
    push_stream: TcpStream,
    req_stream: TcpStream,
    usm: Arc<Mutex<UserManagement>>,
    recv: mpsc::Receiver<CIReturn>,
    sender: mpsc::Sender<CIRequest>
}

pub struct ObserverPool {
    pub observer_pool: HashMap<u64, ObserveConnectionInstance>
}

impl ObserverPool {
    pub fn new() -> Self {
        Self {
            observer_pool: HashMap::new()
        }
    }
    pub fn add_observer(&mut self, oci: ObserveConnectionInstance, uid: u64) {
        self.observer_pool.insert(uid, oci);
    }
}

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
    pub fn observe(&self) -> bool {
        self.push_stream.is_some() && self.req_stream.is_some()
    }
}