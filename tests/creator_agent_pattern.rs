extern crate rust_agents;

use std::collections::BTreeMap;

use rust_agents::behaviour::Behaviour;
use rust_agents::utils::{
    map_tree_leaves, perform_system_actions, AgentBase, AgentId, BaseOp, System, SystemOp,
};

trait ChildGenOp {
    type RequestType;
    fn child_requests(&self) -> Vec<Self::RequestType>;
}

#[derive(Debug, Clone)]
struct RemoveAgentRequest {
    from: AgentId,
    remove: AgentId,
}

struct CreatorBehaviour {}

impl<STATE, CONTEXT, REQUEST> Behaviour<STATE, CONTEXT> for CreatorBehaviour
where
    STATE: BaseOp + ChildGenOp<RequestType = REQUEST> + SystemOp<RequestType = REQUEST> + Clone,
    CONTEXT: ,
    REQUEST: From<RemoveAgentRequest>,
{
    fn act(&self, state: &STATE, _context: &CONTEXT) -> STATE {
        let mut new_state = state.clone();
        for child in state.child_requests() {
            //This should really be child.into()
            new_state.request(child);
        }
        let request = RemoveAgentRequest {
            from: new_state.id(),
            remove: new_state.id(),
        };
        new_state.request(request.into());
        new_state
    }
}

#[derive(Debug, Clone)]
struct CreateAgentRequest {
    new_id: AgentId,
}

#[derive(Debug, Clone)]
enum SystemRequest {
    RemoveAgent(RemoveAgentRequest),
    CreateAgent(CreateAgentRequest),
}

impl From<RemoveAgentRequest> for SystemRequest {
    fn from(v: RemoveAgentRequest) -> Self {
        SystemRequest::RemoveAgent(v)
    }
}

#[derive(Debug, Clone)]
struct CreatorState {
    id: AgentId,
    system_outbox: Vec<SystemRequest>,
}

impl ChildGenOp for CreatorState {
    type RequestType = SystemRequest;
    fn child_requests(&self) -> Vec<Self::RequestType> {
        vec![
            SystemRequest::CreateAgent(CreateAgentRequest { new_id: AgentId(7) }),
            SystemRequest::CreateAgent(CreateAgentRequest { new_id: AgentId(8) }),
        ]
    }
}

impl SystemOp for CreatorState {
    type RequestType = SystemRequest;
    fn request(&mut self, request: Self::RequestType) {
        self.system_outbox.push(request);
    }
}

impl BaseOp for CreatorState {
    fn id(&self) -> AgentId {
        self.id
    }
}

#[derive(Debug, Clone)]
struct ChildState {
    id: AgentId,
}

#[derive(Debug, Clone)]
enum Agent {
    Creator(CreatorState),
    Child(ChildState),
}

impl From<CreatorState> for Agent {
    fn from(v: CreatorState) -> Agent {
        Agent::Creator(v)
    }
}

impl From<ChildState> for Agent {
    fn from(v: ChildState) -> Agent {
        Agent::Child(v)
    }
}

impl Agent {
    pub fn id(&self) -> AgentId {
        match self {
            Agent::Creator(state) => state.id,
            Agent::Child(state) => state.id,
        }
    }

    pub fn act<CONTEXT>(&self, context: &CONTEXT) -> Self {
        match self {
            Agent::Creator(state) => (CreatorBehaviour {}).act(state, context).into(),
            Agent::Child(state) => state.clone().into(), //No child behaviour yet.
        }
    }
}

impl AgentBase<SystemRequest> for Agent {
    fn empty_system_outbox(&mut self) -> Vec<SystemRequest> {
        match self {
            Agent::Creator(state) => state.system_outbox.drain(..).collect(),
            Agent::Child(_) => vec![], //No child behaviour yet.
        }
    }
}

struct GlobalContext {
    agents: BTreeMap<AgentId, Agent>,
}

impl GlobalContext {
    pub fn new() -> Self {
        GlobalContext {
            agents: BTreeMap::new(),
        }
    }
}

fn print_agents(context: &GlobalContext) {
    for (_agent_id, agent) in &context.agents {
        println!("  {:?}", agent)
    }
}

fn step_agents(context: &mut GlobalContext) {
    context.agents = map_tree_leaves(&context.agents, |agent| agent.act(context));
}

// fn gather_messages(context: &mut GlobalContext) -> Vec<Message> {
//     let mut messages = vec![];
//     for (_agent_id, agent) in &mut context.agents {
//         messages.append(&mut agent.state.outbox);
//     }
//     messages
// }

impl System<SystemRequest> for GlobalContext {
    type AgentType = Agent;

    fn apply_system_request(&mut self, action: SystemRequest) {
        match action {
            SystemRequest::RemoveAgent(request) => {
                println!(
                    "{:?} requested removal of {:?}",
                    request.from, request.remove
                );
                let _agent = self.agents.remove(&request.remove).unwrap();
            }
            SystemRequest::CreateAgent(request) => {
                println!("Creation request {:?}", request);
                self.agents.insert(
                    request.new_id,
                    Agent::Child(ChildState { id: request.new_id }),
                );
            }
        }
    }

    fn agents_mut(&mut self) -> Vec<&mut Agent> {
        return self
            .agents
            .iter_mut()
            .map(|(_agent_id, agent)| agent)
            .collect();
    }
}

// fn deliver_messages(context:&mut GlobalContext, messages:Vec<Message>) {
//     for (_agent_id, agent) in &mut context.agents {
//         agent.state.inbox.clear();
//     }
//
//     for message in messages {
//         println!("MESSAGE: {:?}", message);
//         match context.agents.get_mut(&message.to) {
//             Some(agent) => { agent.state.inbox.push(message); },
//             None => { println!("No agent {:?}", message.to); }
//         }
//     }
// }

#[test]
fn test_use_creator_agent_pattern_no_loop() {
    let behaviour = CreatorBehaviour {};
    let state = CreatorState {
        id: AgentId(100),
        system_outbox: vec![],
    };
    let context = ();

    println!("state={:?}", state);

    let new_state = behaviour.act(&state, &context);

    println!("new_state={:?}", new_state);
}

#[test]
fn test_use_creator_agent_pattern() {
    let initial_state: Vec<Agent> = vec![Agent::Creator(CreatorState {
        id: AgentId(1),
        system_outbox: vec![],
    })];

    let mut agents: BTreeMap<AgentId, Agent> = BTreeMap::new();

    for a in initial_state {
        agents.insert(a.id(), a);
    }

    let mut context = GlobalContext::new();
    context.agents = agents;

    for i in 0..10 {
        println!("Step {}", i);
        print_agents(&context);
        step_agents(&mut context);

        // let messages = gather_messages(&mut context);
        perform_system_actions(&mut context);
        // deliver_messages(&mut context, messages);
    }
}
