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
pub struct ReqFriendPage {
	pub wait_list: Vec<CardUnit>,
	uxd: UXD
}

impl ReqFriendPage {
	pub fn new(uxd: UXD) -> Self {
		Self { 
			wait_list: Vec::new(), 
			uxd
		}
	}
	pub fn view(&self) -> iced::Element<'_, RMessage> {
		let mut useres = Vec::new();
		for u in &self.wait_list {
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
	pub fn remove(&mut self, ur: UserRef, mode: CardMode) {
		let idx = self.wait_list.iter().position(|x| x.user == ur && x.mode == mode);
		if let Some(i) = idx {
			self.wait_list.remove(i);
		}
	}
}