use iced::{widget, Task};
use iced::widget::row;
use crate::state::{RMessage, UXD};

#[derive(Clone)]
pub struct Btn {
    pub(crate) name: String,
    pub(crate) msg: RMessage
}

#[derive(Clone)]
pub enum MPopupView {
    BtnOk,
    ShowTextPopup(String, String, Vec<Btn>),
    ShowTextPopupWithOk(String, String),
    ShowTextPopupRed(String, String, Vec<Btn>),
    ShowTextPopupRedWithOk(String, String)
}

#[derive(Clone)]
struct PopupConfig {
    title: String,
    content: String,
    btn: Vec<Btn>,
    red: bool
}

pub struct BasePopup {
    show: bool,
    uxd: UXD,
    title: String,
    content: String,
    btn: Vec<Btn>,
    red: bool,
    queue: Vec<PopupConfig>
}

impl BasePopup {
    pub fn new(uxd: UXD) -> Self {
        Self {
            show: false,
            uxd,
            title: String::new(),
            content: String::new(),
            btn: vec![],
            red: false,
            queue: vec![]
        }
    }
    pub fn view(&self) -> iced::Element<'_, RMessage> {
        let mut btns: Vec<iced::Element<RMessage>> = vec![];
        for i in &self.btn {
            btns.push(widget::button(i.name.as_str()).on_press(i.msg.clone()).into())
        }
        self.show.then(||
            widget::container(
                widget::center(
                    widget::container(
                        widget::column![
                        widget::text(&self.title).size(self.uxd.title_font_size).style(|t| self.red.then(|| widget::text::danger(t)).unwrap_or_else(|| widget::text::primary(t)) ),
                        widget::text(&self.content).size(self.uxd.subtile_font_size),
                        widget::container(
                            widget::row(btns)
                        ).width(iced::Fill).align_x(iced::Right)
                    ].width(iced::Shrink).padding(self.uxd.view_padding).spacing(self.uxd.content_space)
                    )
                        .style(widget::container::bordered_box)
                )
            )
                .style(|_t| widget::container::Style {
                    background: Some(iced::Background::Color(self.uxd.popup_mask_color)),
                    ..widget::container::Style::default()
                })
                .width(iced::Fill).height(iced::Fill).into()
        ).unwrap_or_else(||
            widget::space().into()
        )
    }
    fn set_text_show(&mut self, t: String, c: String) {
        self.title = t;
        self.content = c;
        self.show = true;
    }
    fn btn_ok() -> Vec<Btn> {
        vec![Btn{
            name: "OK".to_string(),
            msg: RMessage::PopupView(MPopupView::BtnOk)
        }]
    }
    fn push_queue(&mut self, cfg: PopupConfig) {
        if self.show {
            self.queue.push(cfg);
        } else {
            self.title = cfg.title;
            self.content = cfg.content;
            self.red = cfg.red;
            self.btn = cfg.btn;
            self.show = true;
        }
    }
    fn show_queue(&mut self) {
        self.show = false;
        if self.queue.len() > 0 {
            let v = self.queue.remove(0);
            self.push_queue(v);
        }
    }
    pub fn update(&mut self, msg: MPopupView) -> iced::Task<RMessage> {
        match msg {
            MPopupView::BtnOk => self.show_queue(),
            MPopupView::ShowTextPopup(t, c, b) => {
                self.push_queue(PopupConfig {
                    title: t,
                    content: c,
                    btn: b,
                    red: false,
                });
            }
            MPopupView::ShowTextPopupWithOk(t, c) => {
                self.push_queue(PopupConfig {
                    title: t,
                    content: c,
                    btn: Self::btn_ok(),
                    red: false,
                });
            }
            MPopupView::ShowTextPopupRed(t, c, b) => {
                self.push_queue(PopupConfig {
                    title: t,
                    content: c,
                    btn: b,
                    red: true,
                });
            }
            MPopupView::ShowTextPopupRedWithOk(t, c) => {
                self.push_queue(PopupConfig {
                    title: t,
                    content: c,
                    btn: Self::btn_ok(),
                    red: true,
                });
            }
        };
        Task::none()
    }
}