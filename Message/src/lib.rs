use std::error::Error;
use std::fmt::{write, Display, Formatter, Debug};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum TargetAddress {
    User(u64),
    Group(u64)
}

#[derive(Deserialize, Serialize)]
pub enum NStream {
    Send(String),
}

#[derive(Deserialize, Serialize, Debug)]
pub enum NReq {
    GetUserFromUid(u64),
    GetAllUsers
}

#[derive(Deserialize, Serialize)]
pub enum NReturnReq {
    GetAllUsers(Vec<UserInfo>),
    GetUserFromUid(ReturnUser)
}

#[derive(Deserialize, Serialize)]
pub enum ReturnUser {
    Ok(UserInfo),
    Error
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UserInfo {
    pub uid: u64,
    pub name: String,
    pub friend: Vec<u64>,
    pub password: String
}

impl PartialEq for UserInfo {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum UserError {
    InvalidUID,
    NotFoundUser
}

impl Display for UserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::InvalidUID => write!(f, "非法UID"),
            UserError::NotFoundUser => write!(f, "未找到指定的用户")
        }
    }
}

impl Error for UserError {}


#[derive(Deserialize, Serialize, Debug)]
pub struct LoginRequest {
    pub uid: u64,
    pub password: String,
    pub name: String,
    pub state: LoginReqState
}

#[derive(Deserialize, Serialize, Debug)]
pub enum LoginReqState {
    SignUp,
    Login,
    ConversionBoost,
    ConversionSendStream(String)
}

#[derive(Deserialize, Serialize)]
pub enum ReturnNLoginReq {
    OK(UserInfo, String),
    UserNotFound,
    PasswordError,
    UserCreateOK(u64),
    ConversionBoostSuccessful,
    ConversionSendStreamSuccessful,
    ConversionBoostFail,
    ConversionSendStreamFail
}

pub fn send_receive<'a, S: Deserialize<'a> + Serialize + Debug>(tcp: &mut TcpStream, send_msg: S) -> String {
    dbg!(&send_msg);
    send(tcp, send_msg);
    receive(tcp)
}

pub fn send<'a, S: Deserialize<'a> + Serialize>(tcp: &mut TcpStream, send_msg: S) {
    let msg = format!("{}\n", serde_json::to_string(&send_msg).unwrap());
    tcp.write_all(msg.as_bytes()).unwrap();
}

pub fn receive(tcp: &TcpStream) -> String {
    let mut ret = String::new();
    match BufReader::new(tcp).read_line(&mut ret) {
        Ok(0) => return "OVER".to_string(),
        Err(_) => return "OVER".to_string(),
        _ => ()
    }
    println!("{ret}");
    ret
}