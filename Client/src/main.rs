mod app;
mod state;
mod login;
mod popup;
mod main_view;
mod user_card;
mod tab;
mod main_search_page;
mod main_req_page;
mod main_chat_with_page;

fn main() -> iced::Result {
    // iced::application(app::LoginView::default, app::LoginView::update, app::LoginView::view)
    //     .run().expect("TODO: panic message");
    println!("ex");
    iced::application(app::RIMessage::default, app::RIMessage::update, app::RIMessage::view)
        .subscription(app::RIMessage::subscription)
        .run()
}
