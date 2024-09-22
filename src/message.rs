#[derive(Debug, PartialEq, Clone)]
pub enum Data {
    Integer(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Signal(String, Data),
    Subscription(String),
}
