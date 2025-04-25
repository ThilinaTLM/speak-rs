use std::time::Instant;

#[derive(Debug, Clone)]
pub enum Message {
    OnBtnRecord,
    Tick(Instant),
}

