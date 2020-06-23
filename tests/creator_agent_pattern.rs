extern crate rust_agents;

use std::collections::BTreeMap;

use rust_agents::behaviour::Behaviour;
use rust_agents::utils::{AgentId, BaseOp};

trait ChildGenOp {
    type RequestType;
    fn child_requests(&self) -> Vec<Self::RequestType>;
}

trait SystemOp {
    type RequestType;
    fn request(&mut self, request: Self::RequestType);
}

#[derive(Debug)]
struct RemoveAgentRequest {
    from: AgentId,
    remove: AgentId,
}

struct CreatorBehaviour {}

impl<STATE, CONTEXT, REQUEST> Behaviour<STATE, CONTEXT> for CreatorBehaviour
where
    STATE: BaseOp + ChildGenOp<RequestType = REQUEST> + SystemOp<RequestType = REQUEST>,
    CONTEXT: ,
    REQUEST: From<RemoveAgentRequest>,
{
    fn act(&self, state: STATE, _context: &CONTEXT) -> STATE {
        let mut state = state;
        for child in state.child_requests() {
            //This should really be child.into()
            state.request(child);
        }
        let request = RemoveAgentRequest {
            from: state.id(),
            remove: state.id(),
        };
        state.request(request.into());
        state
    }
}

#[derive(Debug)]
struct CreateAgentRequest {
    new_id: AgentId,
}

#[derive(Debug)]
enum SystemRequest {
    RemoveAgent(RemoveAgentRequest),
    CreateAgent(CreateAgentRequest),
}

impl From<RemoveAgentRequest> for SystemRequest {
    fn from(v: RemoveAgentRequest) -> Self {
        SystemRequest::RemoveAgent(v)
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
struct ChildState {
    id: AgentId,
}

#[derive(Debug)]
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

    pub fn act<CONTEXT>(self, context: &CONTEXT) -> Self {
        match self {
            Agent::Creator(state) => (CreatorBehaviour {}).act(state, context).into(),
            Agent::Child(state) => state.into(), //No child behaviour yet.
        }
    }

    pub fn empty_system_outbox(&mut self) -> Vec<SystemRequest> {
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
    let mut temp: BTreeMap<AgentId, Agent> = BTreeMap::new();
    std::mem::swap(&mut temp, &mut context.agents);
    context.agents = temp
        .into_iter()
        .map(|(agent_id, agent)| (agent_id, agent.act(context)))
        .collect();
}

// fn gather_messages(context: &mut GlobalContext) -> Vec<Message> {
//     let mut messages = vec![];
//     for (_agent_id, agent) in &mut context.agents {
//         messages.append(&mut agent.state.outbox);
//     }
//     messages
// }

fn perform_system_actions(context: &mut GlobalContext) {
    let mut system_actions = vec![];
    for (_agent_id, agent) in &mut context.agents {
        system_actions.append(&mut agent.empty_system_outbox());
    }

    for action in system_actions {
        match action {
            SystemRequest::RemoveAgent(request) => {
                println!(
                    "{:?} requested removal of {:?}",
                    request.from, request.remove
                );
                let _agent = context.agents.remove(&request.remove).unwrap();
            }
            SystemRequest::CreateAgent(request) => {
                println!("Creation request {:?}", request);
                context.agents.insert(
                    request.new_id,
                    Agent::Child(ChildState { id: request.new_id }),
                );
            }
        }
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

    let new_state = behaviour.act(state, &context);

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
