use std::error::Error;
use std::fmt::{Display, Formatter, Debug};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use serde::{Deserialize, Serialize};

pub type GroupId = u64;
pub type UserId = u64;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum TargetAddress {
    User(UserId),
    Group(GroupId)
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum FromAddress {
    User(UserId),
    Group(GroupId, UserId)
}

#[derive(Deserialize, Serialize, Debug)]
pub enum NPushStream {
    Send(FromAddress, ChatMessage),
    AddFriendReq(UserId),
    SureFriendReq(UserId),
    InviteGroupReq(UserId, GroupRef),
    SureGroupReq(UserId, GroupId),
    // IntoGroup(UserId, GroupId)
}

#[derive(Deserialize, Serialize, Debug)]
pub enum NReqSigle {
    Send(TargetAddress, ChatMessage),
    AddFriendReq(UserId),
    SureFriendReq(UserId),
    InviteGroupReq(UserId, GroupId),
    SureGroupReq(GroupId)
}

#[derive(Deserialize, Serialize, Debug)]
pub enum NReq {
    GetUserFromUid(UserId),
    GetAllUsers,
    Single(NReqSigle),
    CreateGroup(String),
    GetGroupFromId(GroupId)
}


#[derive(Deserialize, Serialize)]
pub enum NReturnReq {
    GetAllUsers(Vec<UserInfo>),
    GetUserFromUid(Option<UserRef>),
    CreateGroup(GroupRef),
    GetGroupFromId(Option<GroupHeader>)
}

pub trait GetFriendUid {
    fn get_friend(&self) -> Vec<UserId>;
    fn get_uid(&self) -> UserId;
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UserInfo {
    pub uid: UserId,
    pub name: String,
    pub friend: Vec<UserRef>,
    pub password: String,
    pub group: Vec<GroupRef>
}

impl UserInfo {
    pub fn add_friend(&mut self, user_ref: UserRef) {
        if !self.friend.contains(&user_ref) {
            self.friend.push(user_ref);
        }
    }
    pub fn remove_friend(&mut self, user_ref: UserRef) {
        if let Some(idx) = self.friend.iter().position(|x| *x == user_ref) {
            self.friend.remove(idx);
        }
    }
    pub fn get_friend(&self) -> Vec<UserRef> {
        self.friend.clone()
    }
    pub fn to_ref(&self) -> UserRef {
        UserRef {
            name: self.name.clone(),
            uid: self.uid,
            friend: self.friend.clone().into_iter().map(|x| x.uid).collect()
        }
    }
    pub fn get_group(&self) -> Vec<GroupRef> {
        self.group.clone()
    }
    pub fn add_group(&mut self, group_ref: GroupRef) {
        if let None = self.group.iter().find(|x| x.id == group_ref.id) {
            self.group.push(group_ref);
        }
    }
}

impl GetFriendUid for UserInfo {
    fn get_friend(&self) -> Vec<UserId> {
        self.friend.clone().into_iter().map(|x| x.uid).collect()
    }
    fn get_uid(&self) -> UserId {
        self.uid
    }
}

impl Default for UserInfo {
    fn default() -> Self {
        Self { uid: 0, name: "".to_string(), friend: Vec::new(), password: "".to_string(), group: Vec::new() }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UserRef {
    pub uid: UserId,
    pub name: String,
    pub friend: Vec<UserId>
}

impl Default for UserRef {
    fn default() -> Self {
        Self { uid: 0, name: "".to_string(), friend: Vec::new() }
    }
}

impl PartialEq for UserRef {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}
impl Eq for UserRef {}
impl GetFriendUid for UserRef {
    fn get_friend(&self) -> Vec<UserId> {
        self.friend.clone()
    }
    fn get_uid(&self) -> UserId {
        self.uid
    }
}
pub fn is_friend(user: &impl GetFriendUid, other: UserId) -> bool {
    for i in &user.get_friend() {
        if *i == other {
            return true;
        }
    }
    if user.get_uid() == other {
        return true
    }
    false
}

impl PartialEq for UserInfo {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum GroupPermission {
    Chat,
    Administrator,
    All
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GroupHeader {
    pub id: GroupId,
    pub name: String,
    pub user: Vec<GroupUnit>
}

impl Default for GroupHeader {
    fn default() -> Self {
        GroupHeader { id: 0, name: String::new(), user: Vec::new() }
    }
}

impl GroupHeader {
    pub fn to_ref(&self) -> GroupRef {
        GroupRef {
            name: self.name.clone(),
            id: self.id.clone()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GroupUnit {
    pub user: UserRef,
    pub permission: GroupPermission
}

impl GroupUnit {
    pub fn new(user: UserRef) -> Self {
        Self {
            user,
            permission: GroupPermission::Chat
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct GroupRef {
    pub name: String,
    pub id: GroupId
}

impl Default for GroupRef {
    fn default() -> Self {
        Self { name: String::new(), id: 0 }
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
    pub uid: UserId,
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
    UserCreateOK(UserId),
    ConversionBoostSuccessful,
    ConversionSendStreamSuccessful,
    ConversionBoostFail,
    ConversionSendStreamFail
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChatMessage {
	pub sender: UserRef,
	pub message: String
}

impl Default for ChatMessage {
	fn default() -> Self {
		Self { sender: UserRef::default(), message: String::new() }
	}
}

impl Display for ChatMessage {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}: {}", self.sender.name, self.message)
	}
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