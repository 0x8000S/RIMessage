use std::sync::{Arc, Mutex};

use Message::{ChatMessage, GroupHeader, TargetAddress, UserRef, is_friend};
use iced::{Task, widget};
use crate::state::{RMessage, UXD};
use crate::user_card::{RecordManagement, chat_text, user_card};


#[derive(Clone, Debug)]
pub enum MMainChat {
	WhenChatMessageType(String),
	WhenGroupMessageType(String),
	Send(TargetAddress, String)
}

#[derive(Debug)]
pub struct ChatWith {
	chat_message: String,
	group_message: String,
	pub talk_user: UserRef,
	pub talk_group: Arc<Mutex<GroupHeader>>,
	pub yo: UserRef,
	uxd: UXD
}

impl ChatWith {
	pub fn new(uxd: UXD) -> Self {
		Self {
			chat_message: String::new(),
			group_message: String::new(),
			talk_user: UserRef::default(),
			talk_group: Arc::new(Mutex::new(GroupHeader::default())),
			yo: UserRef::default(),
			uxd
		}
	}
	pub fn change_user(&mut self, other: UserRef) {
		self.talk_user = other;
	}
	pub fn change_group(&mut self, other: GroupHeader) {
		*self.talk_group.lock().unwrap() = other;
	}
	pub fn chat_view(&self, record: &RecordManagement) -> iced::Element<'_, RMessage> {
		let mut msg = Vec::new();
		let smsg = record.get_chat(self.talk_user.uid);
		for m in smsg {
			msg.push(chat_text(m.clone(), m.sender.uid == self.yo.uid, &self.uxd));
		}
		widget::column![
			widget::text(self.talk_user.name.clone()).size(self.uxd.subtile_font_size),
			widget::scrollable(
				widget::column(
					msg
				).spacing(self.uxd.content_space).padding(self.uxd.view_padding),
			).id("ChatScrView").height(iced::Fill),
			widget::row![
				widget::text_input("键入消息", &self.chat_message).on_input(|x| RMessage::MainView(crate::main_view::MMainView::MainChat(MMainChat::WhenChatMessageType(x)))).on_submit(RMessage::MainView(crate::main_view::MMainView::MainChat(MMainChat::Send(TargetAddress::User(self.talk_user.uid), self.chat_message.clone())))),
				widget::button("发送").on_press(RMessage::MainView(crate::main_view::MMainView::MainChat(MMainChat::Send(TargetAddress::User(self.talk_user.uid), self.chat_message.clone()))))
			].spacing(self.uxd.content_space)
		].padding(self.uxd.view_padding).into()
	}
	pub fn update(&mut self, msg: MMainChat) -> Task<RMessage> {
		match msg {
			MMainChat::WhenChatMessageType(s) => self.chat_message = s,
			MMainChat::WhenGroupMessageType(s) => self.group_message = s,
			MMainChat::Send(t, m) => {
				self.chat_message.clear();
				self.group_message.clear();
				return Task::done(
					RMessage::MainView(crate::main_view::MMainView::Send(t, ChatMessage {
						sender: self.yo.clone(),
						message: m
					}
				))
				)
			}
		}
		Task::none()
	}
	pub fn group_view(&self, record: &RecordManagement) -> iced::Element<'_, RMessage> {
		let mut msg = Vec::new();
		let id = self.talk_group.lock().unwrap().id;
		let user = self.talk_group.lock().unwrap().user.clone();
		let name = self.talk_group.lock().unwrap().name.clone();
		let smsg = record.get_group_chat(id);
		for m in smsg {
			msg.push(chat_text(m.clone(), m.sender.uid == self.yo.uid, &self.uxd));
		}
		let mut gucard = Vec::new();
		for i in &user {
			gucard.push(user_card(i.clone().user, is_friend(&self.yo, i.clone().user.uid), &self.uxd));
		}
		widget::row![
			widget::column![
				widget::text(name).size(self.uxd.subtile_font_size),
				widget::scrollable(
					widget::column(
						msg
					).spacing(self.uxd.content_space).padding(self.uxd.view_padding),
				).id("ChatScrView").height(iced::Fill),
				widget::row![
					widget::text_input("键入消息", &self.group_message).on_input(|x| RMessage::MainView(crate::main_view::MMainView::MainChat(MMainChat::WhenGroupMessageType(x)))).on_submit(RMessage::MainView(crate::main_view::MMainView::MainChat(MMainChat::Send(TargetAddress::Group(id), self.group_message.clone())))),
					widget::button("发送").on_press(RMessage::MainView(crate::main_view::MMainView::MainChat(MMainChat::Send(TargetAddress::Group(id), self.group_message.clone()))))
				].spacing(self.uxd.content_space)
			].width(iced::FillPortion(3)),
			widget::column![
				widget::scrollable(widget::column(gucard)).height(iced::Fill),
				widget::button("添加新成员").width(iced::Fill).on_press(RMessage::MainView(crate::main_view::MMainView::ShowSelectUserMessageBox))
			].width(iced::FillPortion(1))
		].padding(self.uxd.view_padding).spacing(self.uxd.content_space).into()
	}
	pub fn chat_slide_down(&mut self) -> Task<RMessage> {
		iced::widget::operation::snap_to_end("ChatScrView")
	}
}