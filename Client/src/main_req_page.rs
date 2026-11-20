use Message::{GroupRef, UserRef};
use iced::widget;

use crate::{state::{RMessage, UXD}, user_card::{user_friend_req, user_into_group_req}};

#[derive(Debug, PartialEq, PartialOrd)]
pub enum CardMode {
	ReqFriend,
	ReqIntoGroup
}

#[derive(Debug)]
pub struct CardUnit {
	pub user: UserRef,
	pub mode: CardMode,
	pub gr: Option<GroupRef>
}

#[derive(Debug)]
pub struct ReqFriendPageData {
	pub wait_list: Vec<CardUnit>
}

impl ReqFriendPageData {
	pub fn remove(&mut self, ur: u64, mode: CardMode) {
		let idx = match mode {
			CardMode::ReqFriend => {
				self.wait_list.iter().position(|x| x.user.uid == ur && x.mode == mode)
			}
			CardMode::ReqIntoGroup => {
				let mut ret = None;
				for (i, c) in self.wait_list.iter().enumerate() {
					if let CardMode::ReqIntoGroup = c.mode {
						if let Some(x) = &c.gr {
							if x.id == ur {
								ret = Some(i);
							}
						}
					}
				}
				ret
			}
		};
		if let Some(i) = idx {
			self.wait_list.remove(i);
		}
	}
}

#[derive(Debug)]
pub struct ReqFriendPage {
	uxd: UXD
}

impl ReqFriendPage {
	pub fn new(uxd: UXD) -> Self {
		Self { 
			uxd
		}
	}
	pub fn view(&self, wait_list: &Vec<CardUnit>) -> iced::Element<'_, RMessage> {
		let mut useres = Vec::new();
		for u in wait_list {
			match u.mode {
				CardMode::ReqFriend => useres.push(user_friend_req(u.user.clone(), &self.uxd)),
				CardMode::ReqIntoGroup => useres.push(user_into_group_req(u.user.clone(), u.gr.clone().unwrap(), &self.uxd)),
			}
		}
		widget::scrollable(
		widget::column(
			useres
			).padding(self.uxd.view_padding).spacing(self.uxd.content_space)
		).into()
	}
}