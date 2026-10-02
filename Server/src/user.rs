use Message::{UserInfo, UserError, ReturnNLoginReq};

#[derive(Debug)]
pub struct UserManagement {
    user: Vec<UserInfo>
}

impl UserManagement {
    pub fn new() -> Self {
        UserManagement {
            user: vec![]
        }
    }
    pub fn new_user(&self, name: String, password: String) -> UserInfo {
        UserInfo {
            uid: self.user.is_empty().then(|| 1).unwrap_or_else(|| self.user[self.user.len()-1].uid + 1),
            name,
            friend: vec![],
            password
        }
    }
    pub fn add_new_user(&mut self, user: UserInfo) -> ReturnNLoginReq {
        let uid = user.uid;
        self.user.push(user);
        ReturnNLoginReq::UserCreateOK(uid)
    }
    pub fn find_user(&self, uid: u64) -> Option<&UserInfo> {
        self.user.iter().find(|x| x.uid == uid)
    }
    pub fn get_all_users(&mut self) -> Vec<UserInfo> {
        self.user.clone()
    }
}