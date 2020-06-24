extern crate rust_agents;

use rust_agents::behaviour::Behaviour;
use std::collections::BTreeMap;

use rust_agents::utils::{
    map_tree_leaves, perform_system_actions, AgentBase, AgentId, BaseOp, Color, ColorOp, System,
};

trait MessageOp {
    // TODO: Better to use an iterator than allocate?
    // The Message type should be only those that Alice can understand
    fn inbox(&self) -> Vec<Message>;
    fn send(&mut self, message: Message);
}

trait SystemOp {
    fn request(&mut self, request: SystemRequest);
}

trait NameResolver {
    fn resolve_id_from_name(&self, name: &str) -> Option<AgentId>;
}

#[derive(Debug, Clone)]
struct AliceBehaviour {}

impl<STATE, CONTEXT> Behaviour<STATE, CONTEXT> for AliceBehaviour
where
    STATE: BaseOp + ColorOp + MessageOp + SystemOp + Clone,
{
    fn act(&self, state: &STATE, _context: &CONTEXT) -> STATE {
        let mut state = state.clone();

        let inbox = state.inbox();
        let greetings: Vec<&Message> = inbox
            .iter()
            .filter(|m| match m.body {
                MessageBody::Greeting(_) => true,
            })
            .collect();

        if greetings.len() > 0 {
            state.set_color(Color::Blue);
            state.request(SystemRequest {
                from: state.id(),
                body: SystemRequestBody::RemoveAgent(state.id()),
            });
        }

        greetings.iter().for_each(|m| {
            let reply = Message {
                to: m.from,
                from: state.id(),
                body: MessageBody::Greeting(Greeting {
                    msg: "Go away, I’m social-distancing!".to_string(),
                }),
            };
            state.send(reply);
        });

        state
    }
}

#[derive(Debug, Clone)]
struct BobBehaviour {}

impl<STATE, CONTEXT> Behaviour<STATE, CONTEXT> for BobBehaviour
where
    STATE: BaseOp + ColorOp + MessageOp + SystemOp + Clone,
    CONTEXT: NameResolver,
{
    fn act(&self, state: &STATE, context: &CONTEXT) -> STATE {
        let mut state = state.clone();
        let alice_id = context.resolve_id_from_name("Alice");
        match alice_id {
            Some(alice_id) => {
                let message = Message {
                    from: state.id(),
                    to: alice_id,
                    body: MessageBody::Greeting(Greeting {
                        msg: "Hello, Alice.".to_string(),
                    }),
                };
                state.send(message);
            }
            None => {
                state.request(SystemRequest {
                    from: state.id(),
                    body: SystemRequestBody::RemoveAgent(state.id()),
                });
            }
        };

        let inbox = state.inbox();
        let greetings: Vec<&Message> = inbox
            .iter()
            .filter(|m| match m.body {
                MessageBody::Greeting(_) => true,
            })
            .collect();

        if greetings.len() > 0 {
            state.set_color(Color::Red);
        }

        state
    }
}

#[derive(Debug, Clone)]
struct AgentState {
    id: AgentId,
    name: String,
    position: Option<(i32, i32)>,
    color: Color,
    inbox: Vec<Message>,
    outbox: Vec<Message>,
    system_outbox: Vec<SystemRequest>,
}

impl BaseOp for AgentState {
    fn id(&self) -> AgentId {
        self.id
    }
}

impl ColorOp for AgentState {
    fn get_color(&self) -> &Color {
        &self.color
    }
    fn set_color(&mut self, color: Color) {
        self.color = color
    }
}

impl MessageOp for AgentState {
    fn inbox(&self) -> Vec<Message> {
        self.inbox.clone()
    }
    fn send(&mut self, message: Message) {
        self.outbox.push(message)
    }
}

impl SystemOp for AgentState {
    fn request(&mut self, request: SystemRequest) {
        self.system_outbox.push(request);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Greeting {
    msg: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RemoveAgentMessage {
    to_remove: AgentId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MessageBody {
    Greeting(Greeting),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Message {
    to: AgentId,
    from: AgentId,
    body: MessageBody,
}

#[derive(Debug, Clone)]
enum SystemRequestBody {
    RemoveAgent(AgentId),
}

#[derive(Debug, Clone)]
struct SystemRequest {
    from: AgentId,
    body: SystemRequestBody,
}

#[derive(Debug, Clone)]
enum AgentBehaviour {
    Alice(AliceBehaviour),
    Bob(BobBehaviour),
}

impl<STATE, CONTEXT> Behaviour<STATE, CONTEXT> for AgentBehaviour
where
    STATE: BaseOp + ColorOp + MessageOp + SystemOp + Clone,
    CONTEXT: NameResolver,
{
    fn act(&self, state: &STATE, context: &CONTEXT) -> STATE {
        match self {
            AgentBehaviour::Alice(behaviour) => behaviour.act(state, context),
            AgentBehaviour::Bob(behaviour) => behaviour.act(state, context),
        }
    }
}

#[derive(Debug)]
struct Agent {
    behaviour: AgentBehaviour,
    state: AgentState,
}

impl Agent {
    fn act<CONTEXT>(&self, context: &CONTEXT) -> Self
    where
        CONTEXT: NameResolver,
    {
        let behaviour = self.behaviour.clone();
        let state = behaviour.act(&self.state, context);
        Agent { behaviour, state }
    }
}

impl AgentBase<SystemRequest> for Agent {
    fn empty_system_outbox(&mut self) -> Vec<SystemRequest> {
        self.state.system_outbox.drain(..).collect()
    }
}

struct GlobalContext {
    agents: BTreeMap<AgentId, Agent>,
    name_to_agent_id: BTreeMap<String, AgentId>,
}

impl GlobalContext {
    fn new() -> Self {
        GlobalContext {
            agents: BTreeMap::new(),
            name_to_agent_id: BTreeMap::new(),
        }
    }
}

impl NameResolver for GlobalContext {
    fn resolve_id_from_name(&self, name: &str) -> Option<AgentId> {
        self.name_to_agent_id.get(name).cloned()
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

fn gather_messages(context: &mut GlobalContext) -> Vec<Message> {
    let mut messages = vec![];
    for (_agent_id, agent) in &mut context.agents {
        messages.append(&mut agent.state.outbox);
    }
    messages
}

impl System<SystemRequest> for GlobalContext {
    type AgentType = Agent;
    fn apply_system_request(&mut self, request: SystemRequest) {
        println!("ACTION: {:?}", request);
        match request.body {
            SystemRequestBody::RemoveAgent(agent_id) => {
                println!("{:?} requested removal of {:?}", request.from, agent_id);
                let agent = self.agents.remove(&agent_id).unwrap();
                self.name_to_agent_id.remove(&agent.state.name);
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

fn deliver_messages(context: &mut GlobalContext, messages: Vec<Message>) {
    for (_agent_id, agent) in &mut context.agents {
        agent.state.inbox.clear();
    }

    for message in messages {
        println!("MESSAGE: {:?}", message);
        match context.agents.get_mut(&message.to) {
            Some(agent) => {
                agent.state.inbox.push(message);
            }
            None => {
                println!("No agent {:?}", message.to);
            }
        }
    }
}

#[test]
fn test_main() {
    let curstate: Vec<Agent> = vec![
        Agent {
            behaviour: AgentBehaviour::Alice(AliceBehaviour {}),
            state: AgentState {
                id: AgentId(111),
                name: "Alice".to_string(),
                position: Some((0, 0)),
                color: Color::Black,
                inbox: vec![],
                outbox: vec![],
                system_outbox: vec![],
            },
        },
        Agent {
            behaviour: AgentBehaviour::Bob(BobBehaviour {}),
            state: AgentState {
                id: AgentId(222),
                name: "Bob".to_string(),
                position: Some((3, 0)),
                color: Color::Black,
                inbox: vec![],
                outbox: vec![],
                system_outbox: vec![],
            },
        },
    ];

    let mut agents: BTreeMap<AgentId, Agent> = BTreeMap::new();
    let mut name_to_agent_id: BTreeMap<String, AgentId> = BTreeMap::new();

    for a in curstate {
        name_to_agent_id.insert(a.state.name.clone(), a.state.id);
        agents.insert(a.state.id, a);
    }

    let mut context = GlobalContext::new();
    context.name_to_agent_id = name_to_agent_id;
    context.agents = agents;

    for i in 0..10 {
        println!("Step {}", i);
        print_agents(&context);
        step_agents(&mut context);

        let messages = gather_messages(&mut context);
        perform_system_actions(&mut context);
        deliver_messages(&mut context, messages);
    }
}
