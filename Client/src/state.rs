use crate::login::MLoginView;


#[derive(Clone)]
pub enum RMessage {
    LoginView(MLoginView),
    Sync
}

pub enum View {
    Login,
    Main
}

#[derive(Debug)]
pub struct UXD {
    pub view_padding: u32,
    pub content_space: u32
    
}

impl Default for UXD {
    fn default() -> Self {
        Self {
            view_padding: 12,
            content_space: 8
        }
    }
}