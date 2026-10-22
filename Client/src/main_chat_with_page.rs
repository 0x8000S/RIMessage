use std::{collections::HashMap};

use Message::{UserRef, ChatMessage};
use iced::{Task, widget};

use crate::state::{RMessage, UXD};

#[derive(Clone, Debug)]
pub enum MMainChat {
	WhenMessageType(String),
	Send(u64, String)
}


#[derive(Debug)]
pub struct RecordManagement {
	tap: HashMap<u64, Vec<ChatMessage>>,
	space: Vec<ChatMessage>
}

impl RecordManagement {
	fn new() -> Self {
		Self { 
			tap: HashMap::new(),
			space: Vec::new()
		}
	}
	pub fn push_message(&mut self, uid: u64, message: ChatMessage) {
		match self.tap.get_mut(&uid) {
			Some(x) => x.push(message),
			None => {self.tap.insert(uid, vec![message]);}
		};
	}
	fn get_chat(&self, uid: u64) -> &Vec<ChatMessage> {
		match self.tap.get(&uid) {
			Some(x) => x,
			None => &self.space
		}
	}
}

#[derive(Debug)]
pub struct ChatWith {
	message: String,
	pub record: RecordManagement,
	// management: RecordManagement,
	other: UserRef,
	pub yo: UserRef,
	uxd: UXD
}

impl ChatWith {
	pub fn new(uxd: UXD) -> Self {
		Self {
			message: String::new(),
			record: RecordManagement::new(),
			other: UserRef::default(),
			yo: UserRef::default(),
			uxd
		}
	}
	pub fn change_user(&mut self, other: UserRef) {
		self.other = other;

	}
	pub fn view(&self) -> iced::Element<'_, RMessage> {
		let mut msg = Vec::new();
		let smsg = self.record.get_chat(self.other.uid);
		for m in smsg {
			if m.sender.uid == self.yo.uid {
				msg.push(widget::text(m.clone().to_string()).style(widget::text::primary).into());
			} else {
				msg.push(widget::text(m.clone().to_string()).into());
			}
		}
		widget::column![
			widget::text(self.other.name.clone()).size(self.uxd.subtile_font_size),
			widget::scrollable(
				widget::column(
					msg
				),
			).height(iced::Fill),
			widget::row![
				widget::text_input("键入消息", &self.message).on_input(|x| RMessage::MainView(crate::main_view::MMainView::MainChat(MMainChat::WhenMessageType(x)))).on_submit(RMessage::MainView(crate::main_view::MMainView::MainChat(MMainChat::Send(self.other.uid, self.message.clone())))),
				widget::button("发送").on_press(RMessage::MainView(crate::main_view::MMainView::MainChat(MMainChat::Send(self.other.uid, self.message.clone()))))
			].spacing(self.uxd.content_space)
		].padding(self.uxd.view_padding).into()
	}
	pub fn update(&mut self, msg: MMainChat) -> Task<RMessage> {
		match msg {
			MMainChat::WhenMessageType(s) => self.message = s,
			MMainChat::Send(t, m) => {
				self.record.push_message(t, ChatMessage {
						sender: self.yo.clone(),
						message: m.clone()
					});
				self.message.clear();
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
}