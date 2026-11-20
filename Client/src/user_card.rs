use Message::{GroupRef, UserId, UserRef, ChatMessage, GroupId};
use iced::widget;
use crate::state::{RMessage::{self, UserCard}, UXD};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum MUserCard {
    ChatWith(UserRef),
    AddFriend(UserId),
    SureFriend(UserRef),
    ReqFriendNone(UserRef),
    ReqIntoGroupNone(UserRef),
    SureGroup(GroupRef)
}

pub fn user_card(user: UserRef, friend: bool, uxd: &UXD) -> iced::Element<'_, RMessage> {
    let ruser = user.clone();
    widget::container(
widget::row![
            widget::column![
                widget::text(user.name).size(uxd.subtile_font_size),
                widget::text(format!("UID: {}", user.uid)),
                ].spacing(uxd.content_space).padding(uxd.view_padding),
            widget::container(friend
                .then(|| widget::button("聊天").on_press(RMessage::UserCard(MUserCard::ChatWith(ruser.clone()))))
                .unwrap_or_else(|| widget::button("添加好友").on_press(RMessage::UserCard(MUserCard::AddFriend(user.uid))))).padding(uxd.view_padding).width(iced::Fill).align_x(iced::Right)
            ].align_y(iced::Center),
        ).width(iced::Fill).style(widget::container::rounded_box).into()
}

pub fn user_friend_req(user: UserRef, uxd: &UXD) -> iced::Element<'_, RMessage> {
    let userf = user.clone();
    widget::container(
widget::row![
            widget::column![
                widget::text(user.name).size(uxd.subtile_font_size),
                widget::text(format!("UID: {}", user.uid)),
                ].spacing(uxd.content_space).padding(uxd.view_padding),
            widget::container(
                widget::row![
                    widget::button("同意").on_press(RMessage::UserCard(MUserCard::SureFriend(userf.clone()))),
                    widget::button("拒绝").style(widget::button::danger).on_press(UserCard(MUserCard::ReqFriendNone(userf)))
                ].spacing(uxd.content_space)
            ).padding(uxd.view_padding).width(iced::Fill).align_x(iced::Right)
            ].align_y(iced::Center),
        ).width(iced::Fill).style(widget::container::rounded_box).into()
}

pub fn user_into_group_req(user: UserRef, gr: GroupRef, uxd: &UXD) -> iced::Element<'_, RMessage> {
    let userf = user.clone();
    widget::container(
widget::row![
            widget::column![
                widget::text(format!("{}-邀请你加入群聊{}", user.name, gr.name)).size(uxd.subtile_font_size),
                widget::text(format!("GID: {}", gr.id)),
                ].spacing(uxd.content_space).padding(uxd.view_padding),
            widget::container(
                widget::row![
                    widget::button("同意").on_press(RMessage::UserCard(MUserCard::SureGroup(gr.clone()))),
                    widget::button("拒绝").style(widget::button::danger).on_press(UserCard(MUserCard::ReqIntoGroupNone(userf)))
                ].spacing(uxd.content_space)
            ).padding(uxd.view_padding).width(iced::Fill).align_x(iced::Right)
            ].align_y(iced::Center),
        ).width(iced::Fill).style(widget::container::rounded_box).into()
}

#[derive(Debug)]
pub struct RecordManagement {
	tap: HashMap<UserId, Vec<ChatMessage>>,
	group: HashMap<GroupId, Vec<ChatMessage>>,
	space: Vec<ChatMessage>
}

impl RecordManagement {
	pub fn new() -> Self {
		Self { 
			tap: HashMap::new(),
			group: HashMap::new(),
			space: Vec::new()
		}
	}
	pub fn push_message(&mut self, uid: UserId, message: ChatMessage) {
		match self.tap.get_mut(&uid) {
			Some(x) => x.push(message),
			None => {self.tap.insert(uid, vec![message]);}
		};
	}
	pub fn get_chat(&self, uid: UserId) -> &Vec<ChatMessage> {
		match self.tap.get(&uid) {
			Some(x) => x,
			None => &self.space
		}
	}
	pub fn push_group_message(&mut self, uid: GroupId, message: ChatMessage) {
		match self.group.get_mut(&uid) {
			Some(x) => x.push(message),
			None => {self.group.insert(uid, vec![message]);}
		};
	}
	pub fn get_group_chat(&self, uid: GroupId) -> &Vec<ChatMessage> {
		match self.group.get(&uid) {
			Some(x) => x,
			None => &self.space
		}
	}
}

pub fn chat_text(msg: ChatMessage, me: bool, uxd: &UXD) -> iced::Element<'static, RMessage> {
	let c = widget::container(
		widget::text(msg.message)
	).style(move |x: &iced::Theme| me.then(||widget::container::primary(x)).unwrap_or_else(|| widget::container::bordered_box(x)))
	.padding(uxd.view_padding)
	.width(iced::Shrink);
	if me {
		return widget::column(
			[
				widget::text(msg.sender.name.clone()).align_x(iced::Right).style(widget::text::secondary).into(),
				widget::container(c).width(iced::Fill).align_x(iced::Right).into()
			]
		).align_x(iced::Right).into()
	}
	widget::column(
		[
			widget::text(msg.sender.name.clone()).align_x(iced::Left).style(widget::text::secondary).into(),
			widget::container(c).width(iced::Fill).align_x(iced::Left).into()
		]
	).into()
}