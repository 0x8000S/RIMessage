use Message::{UserId, UserInfo};
use iced::{Task, widget};

use crate::{main_view::MMainView::MUserSelect, popup::base, state::{RMessage, UXD}};

#[derive(Clone, Debug)]
pub enum MMainUserSelect {
	Select(UserId),
	Close
}

#[derive(Clone, Debug)]
pub struct UserSelect {
	pub select: UserId,
	uxd: UXD,
	pub show: bool,
	pub user: UserInfo,
	callback: RMessage
}

impl UserSelect {
	pub fn new(uxd: UXD) -> Self {
		Self { select: 0, uxd, show: false, user: UserInfo::default(), callback: RMessage::Tick }
	}
	fn base(&self, user: &UserInfo) -> iced::Element<'_, RMessage> {
		let mut uc = Vec::new();
		for i in user.clone().friend {
			uc.push(
				widget::button(widget::text(i.clone().name.clone()))
				.on_press(RMessage::MainView(MUserSelect(MMainUserSelect::Select(i.uid))))
				.into());
		}
		let w = widget::column![
			widget::text("选择用户").size(self.uxd.title_font_size),
			widget::scrollable(
				widget::grid(
					uc
				).spacing(self.uxd.content_space).fluid(180)
			),
			widget::button("关闭").on_press(RMessage::MainView(MUserSelect(MMainUserSelect::Close)))
		].padding(self.uxd.view_padding).spacing(self.uxd.content_space);
		base(w, self.uxd.clone())
	}
	pub fn view(&self) -> iced::Element<'_, RMessage> {
		if self.show {
			self.base(&self.user)
		} else {
			widget::space().into()
		}
	}
	pub fn update(&mut self, msg: MMainUserSelect) -> Task<RMessage> {
		match msg {
			MMainUserSelect::Select(u) => {
				self.select = u;
				self.show = false;
				return Task::done(self.callback.clone())
			},
			MMainUserSelect::Close => self.show = false
		}
		Task::none()
	}
	pub fn set_callback(&mut self, msg: RMessage) {
		self.callback = msg;
	}
}