mod app;
mod state;
mod login;

fn main() -> iced::Result {
    // iced::application(app::LoginView::default, app::LoginView::update, app::LoginView::view)
    //     .run().expect("TODO: panic message");
    println!("ex");
    iced::application(app::RIMessage::default, app::RIMessage::update, app::RIMessage::view)
        .subscription(app::RIMessage::subscription)
        .run()
}
