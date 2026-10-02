use iced::widget;
use crate::state::{RMessage, UXD};

#[derive(Clone)]
pub enum MUserCard {
    ChatWith(u64),
    AddFriend(u64)
}

pub fn user_card(uid: u64, name: String, friend: bool, uxd: &UXD) -> iced::Element<'_, RMessage> {
    widget::container(
            widget::column![
                widget::text(name).size(uxd.subtile_font_size),
                widget::text(uid),
                friend
                    .then(|| widget::button("聊天").on_press(RMessage::UserCard(MUserCard::ChatWith(uid))))
                    .unwrap_or_else(|| widget::button("添加好友").on_press(RMessage::UserCard(MUserCard::AddFriend(uid))))
            ].spacing(uxd.content_space).padding(uxd.view_padding)
        ).style(widget::container::rounded_box).into()
}
