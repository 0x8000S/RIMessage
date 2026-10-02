mod user;
mod connect_instance;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use user::UserManagement;
use Message::{LoginRequest, ReturnNLoginReq, receive, send, LoginReqState};
use crate::connect_instance::{ObserveConnectionInstance, ObserverPool, OnlinePool};

struct Server {
    online_pool: Arc<Mutex<OnlinePool>>,
    usm: Arc<Mutex<UserManagement>>,
    tcp_listener: TcpListener,
    observer_pool: Arc<Mutex<ObserverPool>>
}

#[derive(Debug)]
struct ServerFA {
    tcp: TcpStream,
    usm: Arc<Mutex<UserManagement>>,
    op: Arc<Mutex<ObserverPool>>,
    olp: Arc<Mutex<OnlinePool>>
}

impl Server {
    fn new() -> Self {
        let usm = Arc::new(Mutex::new(UserManagement::new()));
        Self {
            online_pool: Arc::new(Mutex::new(OnlinePool::new(usm.clone()))),
            usm,
            tcp_listener: TcpListener::bind("127.0.0.1:7878").unwrap(),
            observer_pool: Arc::new(Mutex::new(ObserverPool::new()))
        }
    }
    fn listener(&mut self) {
        let opc = self.observer_pool.clone();
        let olpc = self.online_pool.clone();
        thread::spawn(move || {
            loop {
                opc.lock().unwrap().watch(olpc.clone());
                sleep(Duration::from_millis(800));
            }
        });
        let oplcc = self.online_pool.clone();
        thread::spawn(move || {
            loop {
                // println!("START CLEAR");
                oplcc.lock().unwrap().message_stream();
                // println!("FINAL CLEAR");
                sleep(Duration::from_secs(1));
            }
        });
        for i in self.tcp_listener.incoming() {
            println!("新连接");
            let i = i.unwrap();
            let usmc = self.usm.clone();
            let opc = self.observer_pool.clone();
            let olpc = self.online_pool.clone();
            thread::spawn(move || {
                Server::handling_request(ServerFA {
                    tcp: i,
                    usm: usmc,
                    op: opc,
                    olp: olpc
                })
            });
        }
    }
    fn handling_request(mut sfa: ServerFA) {
        // let sfac = sfa.clone();
        loop {
            let msg = receive(&sfa.tcp);
            if msg == "OVER".to_string() {
                return;
            }
            println!("收到请求: {msg}");
            if msg.is_empty() {
                continue;
            }
            let req: LoginRequest = serde_json::from_str(msg.trim()).unwrap();
            match &req.state {
                LoginReqState::Login => Server::login(&req, &mut sfa),
                LoginReqState::SignUp => Server::signup(&req, &mut sfa),
                LoginReqState::ConversionBoost => {
                    if let Some(o) = &mut sfa.op.lock().unwrap().find_observer(req.uid) {
                        send(&mut sfa.tcp, ReturnNLoginReq::ConversionBoostSuccessful);
                        o.req_stream = Some(sfa.tcp);
                        return;
                    }
                    send(&mut sfa.tcp, ReturnNLoginReq::ConversionBoostFail);
                },
                LoginReqState::ConversionSendStream(token) => {
                    if let Some(o) = sfa.op.lock().unwrap().find_observer(req.uid) {
                        if o.token == *token {
                            send(&mut sfa.tcp, ReturnNLoginReq::ConversionSendStreamSuccessful);
                            o.push_stream = Some(sfa.tcp);
                            if o.observe() {
                                println!("UID={} 观察池已成功匹配", req.uid);
                            }
                            return;
                        }
                        send(&mut sfa.tcp, ReturnNLoginReq::ConversionSendStreamFail);

                    }
                }
            };
            println!("完成请求处理!");
        }
    }
    pub fn login(req: &LoginRequest, sfa: &mut ServerFA) {
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
        send(&mut sfa.tcp, msg);
    }
    pub fn signup(req: &LoginRequest, sfa: &mut ServerFA) {
        let ui = sfa.usm.lock().unwrap().new_user(req.name.clone(), req.password.clone());
        send(&mut sfa.tcp, sfa.usm.lock().unwrap().add_new_user(ui));
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
