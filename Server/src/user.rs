use Message::{GroupHeader, GroupId, GroupPermission, GroupRef, GroupUnit, ReturnNLoginReq, UserId, UserInfo, UserRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserManagement {
    user: Vec<UserInfo>,
    group: Vec<GroupHeader>
}

impl UserManagement {
    pub fn new() -> Self {
        UserManagement {
            user: vec![],
            group: Vec::new()
        }
    }
    pub fn new_user(&self, name: String, password: String) -> UserInfo {
        UserInfo {
            uid: self.user.is_empty().then(|| 1).unwrap_or_else(|| self.user[self.user.len()-1].uid + 1),
            name,
            friend: Vec::new(),
            password,
            group: Vec::new()
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
    pub fn get_mut_user(&mut self, uid: UserId) -> Option<&mut UserInfo> {
        let index = self.user.iter().position(|x| x.uid == uid);
        match index {
            Some(i) => Some(self.user.get_mut(i).unwrap()),
            None => None
        }
    }
    pub fn find_user_ref(&self, uid: UserId) -> Option<UserRef> {
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
    pub fn change_friend(&mut self, p: UserId, c: UserId, state: bool) {
        if let Some(a) = self.find_user_ref(c) {
            if let Some(b) = self.find_user_ref(p) {
                if state {
                    self.get_mut_user(p).unwrap().add_friend(a);
                    self.get_mut_user(c).unwrap().add_friend(b);
                } else {
                    self.get_mut_user(p).unwrap().remove_friend(a);
                    self.get_mut_user(c).unwrap().remove_friend(b);
                }
            }
        }
    }
    pub fn get_all_user_ref(&mut self) -> Vec<UserRef> {
        self.user.iter().map(|x| UserRef {
            uid: x.uid,
            name: x.name.clone()
        }).collect()
    }
    pub fn create_group(&mut self, boss: UserId, name: String) -> GroupRef {
        if let Some(x) = self.user.iter_mut().find(|a| a.uid == boss) {
            let idx = self.group.is_empty().then(|| 1).unwrap_or_else(|| self.group[self.group.len()-1].id + 1);
            x.group.push(GroupRef {
                name: name.clone(),
                id: idx
            });
            self.group.push(
                GroupHeader {
                id: idx,
                name: name.clone(),
                user: vec![
                    GroupUnit {
                            user: x.to_ref(),
                            permission: GroupPermission::All
                        }
                    ]
                }      
            );
            return GroupRef {
                name: name.clone(),
                id: idx
            }
        }
        GroupRef::default()
    }
    pub fn add_unit(&mut self, group: GroupId, unit: UserId) {
        if let Some(x) = self.group.iter_mut().find(|x| x.id == group) {
            if let Some(u) = self.user.iter_mut().find(|a| a.uid == unit) {
                u.group.push(x.to_ref());
                x.user.push(GroupUnit {
                    user: u.to_ref(),
                    permission: GroupPermission::Chat
                });
            }
        }
    }
    pub fn change_unit_permission(&mut self, group: GroupId, user: UserId, state: GroupPermission) {
        if let Some(x) = self.group.iter_mut().find(|x| group == x.id) {
            if let Some(unit) = x.user.iter_mut().find(|z| z.user.uid == user) {
                unit.permission = state;
            }
        }
    }
    pub fn get_unit(&mut self, group: GroupId, user: UserId) -> Option<GroupUnit> {
        if let Some(x) = self.group.iter_mut().find(|x| group == x.id) {
            if let Some(unit) = x.user.iter_mut().find(|z| z.user.uid == user) {
                return Some(unit.clone());
            }
        }
        None
    }
    pub fn remove_group(&mut self, group: GroupId) {
        if let Some(idx) = self.group.iter().position(|x| x.id == group) {
            self.group.remove(idx);
        }
    }
    pub fn get_group_ref(&self, group: GroupId) -> Option<GroupRef> {
        if let Some(gr) = self.group.iter().find(|x| x.id == group) {
            return Some(gr.to_ref())
        }
        None
    }
    pub fn get_group_member(&self, group: GroupId) -> Option<Vec<u64>> {
        if let Some(gh) = self.group.iter().find(|x| x.id == group) {
            let ret: Vec<u64> = gh.user.iter().map(|x| x.user.uid).collect();
            return Some(ret);
        }
        None
    }
    pub fn get_group_administrator(&self, group: GroupId) -> Option<Vec<u64>> {
        if let Some(gh) = self.group.iter().find(|x| x.id == group) {
            let ret: Vec<u64> = gh.user.iter().filter(|x| x.permission == GroupPermission::All || x.permission == GroupPermission::Administrator).map(|x| x.user.uid).collect();
            return Some(ret);
        }
        None
    }
}

