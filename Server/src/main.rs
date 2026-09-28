mod user;
mod connect_instance;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use user::UserManagement;
use Message::{NReqState, UserError, NRequestMessage, LoginRequest, ReturnNLoginReq, receive, send, LoginReqState};
use crate::connect_instance::{ObserveConnectionInstance, ObserverPool, OnlinePool};

struct Server {
    online_pool: Arc<Mutex<OnlinePool>>,
    usm: Arc<Mutex<UserManagement>>,
    tcp_listener: TcpListener,
    observer_pool: Arc<Mutex<ObserverPool>>
}

struct ServerFA {
    tcp: Mutex<TcpStream>,
    usm: Arc<Mutex<UserManagement>>,
    op: Arc<Mutex<ObserverPool>>
}

impl Server {
    fn new() -> Self {
        Self {
            online_pool: Arc::new(Mutex::new(OnlinePool::new())),
            usm: Arc::new(Mutex::new(UserManagement::new())),
            tcp_listener: TcpListener::bind("127.0.0.1:7878").unwrap(),
            observer_pool: Arc::new(Mutex::new(ObserverPool::new()))
        }
    }
    fn listener(&mut self) {
        for i in self.tcp_listener.incoming() {
            println!("新连接");
            let i = i.unwrap();
            let usmc = self.usm.clone();
            let opc = self.observer_pool.clone();
            thread::spawn(move || {
                Server::handling_request(Arc::new(ServerFA {
                    tcp: Mutex::new(i),
                    usm: usmc,
                    op: opc
                }))
            });
        }
    }
    fn handling_request(sfa: Arc<ServerFA>) {
        let sfac = sfa.clone();
        loop {
            let msg = receive(&sfac.tcp.lock().unwrap());
            println!("收到请求: {msg}");
            let req: LoginRequest = serde_json::from_str(msg.trim()).unwrap();
            match &req.state {
                LoginReqState::Login => Server::login(&req, sfac.clone()),
                LoginReqState::SignUp => Server::signup(&req, sfac.clone()),
                _ => ()
            }
        }
    }
    pub fn login(req: &LoginRequest, mut sfa: Arc<ServerFA>) {
        let sfac = sfa.clone();
        let msg = {
            if let Some(u) = sfa.usm.lock().unwrap().find_user(req.uid) {
                if u.password == req.password {
                    let oci = ObserveConnectionInstance::new();
                    let token = oci.token.clone();
                    sfa.op.lock().unwrap().add_observer(oci, u.uid);
                    ReturnNLoginReq::OK(u.clone(), token)
                } else {
                    ReturnNLoginReq::PasswordError
                }
            } else {
                ReturnNLoginReq::UserNotFound
            }
        };
        send(&mut sfa.tcp.lock().unwrap(), msg);
    }
    pub fn signup(req: &LoginRequest, mut sfa: Arc<ServerFA>) {
        let ui = sfa.usm.lock().unwrap().new_user(req.name.clone(), req.password.clone());
        send(&mut sfa.tcp.lock().unwrap(), sfa.usm.lock().unwrap().add_new_user(ui));
    }
}

fn main() {
    let mut sver = Server::new();
    sver.listener();
    // let tcpl = TcpListener::bind("127.0.0.1:7878").unwrap();
    // let mut usm = UserManagement::new();
    // usm.add_new_user(usm.new_user("Alice".to_string(), String::from("1234")));
    // usm.add_new_user(usm.new_user("Elik".to_string(), String::from("5543")));
    // let mut i = tcpl.accept().unwrap().0;
    // // for z in 0..41 {
    // //     let mut m = z.to_string();
    // //     m.push_str("\n");
    // //     i.write_all(m.as_ref()).unwrap();
    // //     sleep(Duration::from_secs(1))
    // // }
    // println!("Got!");
    // let mut msg = String::new();
    // BufReader::new(&i).read_line(&mut msg).expect("TODO: panic message");
    // println!("{}", msg.trim());

    // let ret = handling_request(&mut usm, serde_json::from_str(msg.as_str()).expect("REASON"));
    // let rw = Message::NMessage {
    //     sender: 0,
    //     receiver: Message::TargetAddress::User(0),
    //     state: State::Return(ret)
    // };
    // i.write_all(serde_json::to_string(&rw).unwrap().as_bytes()).unwrap();

}
//
// fn handling_request(usm: &mut UserManagement, msg: NRequestMessage) -> ReturnNMessage {
//     match &msg.state {
//         NReqState::CreateNewUser(name, p) => {
//             usm.add_new_user(usm.new_user(name.to_string(), p.to_string()))
//         }
//         NReqState::Return(x) => ReturnNMessage::OK
//     }
// }
