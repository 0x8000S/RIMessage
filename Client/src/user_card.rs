use Message::UserRef;
use iced::widget;
use crate::state::{RMessage::{self, UserCard}, UXD};

#[derive(Clone, Debug)]
pub enum MUserCard {
    ChatWith(UserRef),
    AddFriend(u64),
    SureFriend(UserRef),
    None(UserRef)
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
                    widget::button("拒绝").style(widget::button::danger).on_press(UserCard(MUserCard::None(userf)))
                ].spacing(uxd.content_space)
            ).padding(uxd.view_padding).width(iced::Fill).align_x(iced::Right)
            ].align_y(iced::Center),
        ).width(iced::Fill).style(widget::container::rounded_box).into()
}