#[derive(Debug, Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct AgentId(pub u64);

pub trait BaseOp {
    fn id(&self) -> AgentId;
}

pub trait System<T> {
    fn apply_system_request(&mut self, action: T);
}

#[derive(Debug)]
pub enum Color {
    Black,
    Blue,
    Red,
}

pub trait ColorOp {
    fn get_color(&self) -> &Color;
    fn set_color(&mut self, color: Color);
}
