use crate::lang::LangMessage;

pub trait MessageProvider: Sync + Send {
    fn set_message(&self, message: LangMessage);
    fn get_message(&self) -> Option<LangMessage>;
    fn clear(&self);
    fn request_offline_nickname(&mut self) -> String;
    fn need_offline_nickname(&self) -> bool;
    fn set_offline_nickname(&mut self, nickname: String);
}
