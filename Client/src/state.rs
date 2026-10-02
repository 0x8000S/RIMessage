use std::net::TcpStream;
use Message::UserInfo;
use crate::login::MLoginView;
use crate::main_view::MMainView;
use crate::popup::MPopupView;
use crate::user_card::MUserCard;

#[derive(Clone)]
pub enum RMessage {
    LoginView(MLoginView),
    PopupView(MPopupView),
    Sync,
    Login(UserInfo),
    MainView(MMainView),
    Exit,
    UserCard(MUserCard)
}

pub enum View {
    Login,
    Main
}

#[derive(Debug)]
pub struct UXD {
    pub view_padding: f32,
    pub content_space: u32,
    pub popup_mask_color: iced::Color,
    pub title_font_size: f32,
    pub subtile_font_size: f32
    
}

impl Default for UXD {
    fn default() -> Self {
        Self {
            view_padding: 12.0,
            content_space: 8,
            popup_mask_color: iced::Color::from_rgba8(0, 0, 0, 0.6),
            title_font_size: 48.0,
            subtile_font_size: 28.0
        }
    }
}