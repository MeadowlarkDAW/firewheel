#[derive(Debug)]
pub enum Message<C: std::fmt::Debug> {
    Custom(C),
}
