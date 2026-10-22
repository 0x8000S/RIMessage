use Message::UserRef;
use iced::widget;

use crate::{state::{RMessage, UXD}, user_card::user_friend_req};


#[derive(Debug)]
pub struct ReqFriendPage {
	pub wait_list: Vec<UserRef>,
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
			useres.push(user_friend_req(u.clone(), &self.uxd));
		}
		widget::scrollable(
		widget::column(
			useres
			).padding(self.uxd.view_padding).spacing(self.uxd.content_space)
		).into()
	}
}