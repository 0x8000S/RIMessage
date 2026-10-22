use crate::{main_view::MMainView::{MSearchPage, SearchUser}, state::{RMessage, UXD}, user_card::user_card};
use Message::{UserInfo, UserRef, is_friend};
use iced::{Task, widget};


#[derive(Clone, Debug)]
pub enum MMainSearchPage {
	WhenSearchBoxType(String),
	Search
}

#[derive(Debug)]
pub struct SearchPage {
	search_uid: String,
	pub search_card: Vec<UserRef>,
	pub user_info: UserInfo,
	uxd: UXD
}

impl SearchPage {
	pub fn new(user_info: UserInfo, uxd: UXD) -> Self {
		Self {
			search_uid: String::new(),
			search_card: Vec::new(),
			user_info: user_info,
			uxd: uxd
		}
	}
	pub fn view(&self) -> iced::Element<'_, RMessage> {
        let mut test = vec![];
        for ur in &self.search_card {
            test.push(user_card(ur.clone(), is_friend(&self.user_info, self.search_uid.parse().unwrap_or_else(|_| 0)), &self.uxd));
        }
        widget::column![
            widget::row![
                widget::text_input("键入UID", &self.search_uid).on_input(|x| RMessage::MainView(MSearchPage(MMainSearchPage::WhenSearchBoxType(x)))),
                widget::button("搜索").on_press(RMessage::MainView(MSearchPage(MMainSearchPage::Search)))
            ].spacing(self.uxd.content_space),
            widget::scrollable(
                widget::column(
                    test
                ).width(iced::Fill).spacing(self.uxd.content_space)
            ).spacing(self.uxd.content_space)
        ].spacing(self.uxd.content_space).padding(self.uxd.view_padding).into()
    }
	pub fn update(&mut self, msg: MMainSearchPage) -> Task<RMessage> {
		match msg {
			MMainSearchPage::WhenSearchBoxType(t) => {
				if let Ok(_) = t.parse::<u64>() {
					self.search_uid = t
				} else {
					if t.is_empty() {
						self.search_uid.clear();
					}
				}
			},
			MMainSearchPage::Search => {
				self.search_card.clear();
				return Task::done(RMessage::MainView(SearchUser(self.search_uid.parse().unwrap_or_else(|_x| 0))))
			}
		}
		Task::none()
	}
}