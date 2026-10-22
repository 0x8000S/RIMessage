use Message::{ReturnNLoginReq, UserInfo, UserRef};

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
    pub fn get_mut_user(&mut self, uid: u64) -> Option<&mut UserInfo> {
        let index = self.user.iter().position(|x| x.uid == uid);
        match index {
            Some(i) => Some(self.user.get_mut(i).unwrap()),
            None => None
        }
    }
    pub fn find_user_ref(&self, uid: u64) -> Option<UserRef> {
        if let Some(x) = self.user.iter().find(|x| x.uid == uid) {
            Some(UserRef {
                name: x.name.clone(),
                uid: uid
            })
        } else {
            None
        }
    }
    pub fn get_all_users(&mut self) -> Vec<UserInfo> {
        self.user.clone()
    }
    pub fn add_friend(&mut self, p: u64, c: u64) {
        let a = self.find_user_ref(c).unwrap().clone();
        self.get_mut_user(p).unwrap().add_friend(a);
    }
}