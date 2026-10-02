mod app;
mod state;
mod login;
mod popup;
mod main_view;
mod main_view_sidebar;
mod user_card;
mod tab;

fn main() -> iced::Result {
    // iced::application(app::LoginView::default, app::LoginView::update, app::LoginView::view)
    //     .run().expect("TODO: panic message");
    println!("ex");
    iced::application(app::RIMessage::default, app::RIMessage::update, app::RIMessage::view)
        .subscription(app::RIMessage::subscription)
        .run()
}
