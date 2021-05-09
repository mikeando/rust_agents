use std::collections::BTreeMap;

use crate::utils::AgentId;

pub trait MapContext<AGENT> {
    fn set_agents(&mut self, agents: BTreeMap<AgentId, AGENT>);
    fn agents(&self) -> &BTreeMap<AgentId, AGENT>;
}

pub struct SimpleMapContext<AGENT> {
    pub agents: BTreeMap<AgentId, AGENT>,
}

impl<AGENT> SimpleMapContext<AGENT> {
    pub fn new() -> SimpleMapContext<AGENT> {
        SimpleMapContext {
            agents: BTreeMap::new(),
        }
    }
}

impl<AGENT> MapContext<AGENT> for SimpleMapContext<AGENT> {
    fn set_agents(&mut self, agents: BTreeMap<AgentId, AGENT>) {
        self.agents = agents;
    }

    fn agents(&self) -> &BTreeMap<AgentId, AGENT> {
        &self.agents
    }
}
