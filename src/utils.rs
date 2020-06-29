#[derive(Debug, Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct AgentId(pub u64);

pub trait BaseOp {
    fn id(&self) -> AgentId;
}

pub trait AgentBase<REQUEST> {
    fn empty_system_outbox(&mut self) -> Vec<REQUEST>;
}

pub trait System<REQUEST> {
    type AgentType;
    fn apply_system_request(&mut self, action: REQUEST);
    fn agents_mut(&mut self) -> Vec<&mut Self::AgentType>;
}

pub fn perform_system_actions<T, REQUEST>(context: &mut T)
where
    T: System<REQUEST>,
    T::AgentType: AgentBase<REQUEST>,
{
    // We do this is two loops as we can't modify the
    // context while iterating over part of it.
    let mut system_actions = vec![];
    for agent in &mut context.agents_mut() {
        system_actions.append(&mut agent.empty_system_outbox());
    }

    for action in system_actions {
        context.apply_system_request(action);
    }
}

#[derive(Debug, Clone)]
pub enum Color {
    Black,
    Blue,
    Red,
}

pub trait ColorOp {
    fn get_color(&self) -> &Color;
    fn set_color(&mut self, color: Color);
}

pub trait SystemOp {
    type RequestType;
    fn request(&mut self, request: Self::RequestType);
}

use std::collections::BTreeMap;

pub fn map_tree_leaves<A, B, C, F>(tree: &BTreeMap<A, B>, f: F) -> BTreeMap<A, C>
where
    F: Fn(&B) -> C,
    A: Ord + Clone,
{
    tree.iter().map(|(k, v)| (k.clone(), f(v))).collect()
}
