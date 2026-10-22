use iced::{Task, widget};

use crate::{state::{RMessage, UXD}};

#[derive(Clone, Debug)]
pub enum MTabView {
	SwitchView(usize, Box<RMessage>)
}

#[derive(Clone, Debug)]
pub struct TabItem {
	pub name: String,
	pub message: RMessage,
}

fn tab_button(tab: TabItem, index: usize, set: bool, uxd: UXD) -> iced::Element<'static, RMessage> {
	widget::button(
		set.then(|| widget::text(tab.name.clone()).style(widget::text::primary).size(uxd.title_font_size))
		.unwrap_or_else(|| widget::text(tab.name).style(widget::text::base).size(uxd.subtile_font_size))
	).on_press(RMessage::TabView(MTabView::SwitchView(index, Box::from(tab.message))))
		.style(widget::button::text)
		.into()
}

#[derive(Debug)]
pub struct TabView {
	items: Vec<TabItem>,
	now_set: usize,
	uxd: UXD
}

impl TabView {
	pub fn new(items: Vec<TabItem>, uxd: UXD) -> Self {
		TabView { items, now_set: 0, uxd }
	}
	pub fn view(&self) -> iced::Element<'_, RMessage> {
		let mut index = self.now_set;
		if index > self.items.len() {
			index = 0;
		}
		if self.items.len() == 0 {
			widget::space().into()
		} else {
			let mut btns = Vec::new();
			for (i, t) in self.items.iter().enumerate() {
				btns.push(tab_button(t.clone(), i, i == index, self.uxd.clone()));
			}
			widget::container(
				widget::row(btns).align_y(iced::Bottom)
			).into()
		}
	}
	pub fn update(&mut self, msg: MTabView) -> Task<RMessage> {
		match msg {
			MTabView::SwitchView(i, m) => {
				self.now_set = i;
				Task::done(*m)
			}
		}
	}
}