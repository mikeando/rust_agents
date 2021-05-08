extern crate rust_agents;

use rust_agents::behaviour::Behaviour;
use std::collections::BTreeMap;

use rust_agents::utils::{
    map_tree_leaves, perform_system_actions, AgentBase, AgentId, BaseOp, Color, ColorOp, System,
};

/// Stateless behaviour for Alice.
#[derive(Debug, Clone)]
struct AliceBehaviour {}

/// Alice's behaviour implements the Behaviour trait.
///
/// Alice checks if she has received a greeting, and if so turns blue
/// and removes herself from the simulation in the next time-step.
/// she also replies to each message with a greeting of
/// 'Go away, I’m social-distancing!'.
///
/// We want the AliceBehaviour to be able to work on any
/// STATE and CONTEXT where the above set of operations makes
/// sense.
///
/// We cont actually use the any context so that has no trait restrictions.
//
/// However the state (which includes the inbox) has the following requirements:
/// * we must be able to clone the state to get the next state so that the new
///   state is completely independent of the old state - this means we must support Clone
/// * get incoming messages and sending messages - must support MessageOp trait.
/// * set the agents colour - ColorOp
/// * remove its self from simulation - SystemOp
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

/// Stateless behaviour for Bob.
#[derive(Debug, Clone)]
struct BobBehaviour {}

/// Bob's behaviour implements the Behaviour trait.
///
/// Bob sends alice a message saying 'Hello, Alice'.
/// Then waits for a response and turns red upon receiving one.
///
/// As for the AliceBehaviour we want this to work on any suitable
/// STATE and CONTEXT pairs.
///
/// In Bob's case we do need a way to find Alice - in this case that
/// means we want the CONTEXT to support a way to get an AgentId from
/// a name - so we require CONTEXT to implement a new NameResolver trait.
///
/// Bob also needs to change colour, receive messages and remove himsefl
/// from the simulation, meaning we require the STATE to implement
/// ColorOpt, MessageOp, SystemOp as well as the "usual" BaseOp and Clone.
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

/// A trait for states that send and receive messages
/// In this case hardcoded to receive alice_bob::Message
trait MessageOp {
    // TODO: Better to use an iterator than allocate?
    // The Message type should be only those that Alice can understand
    fn inbox(&self) -> Vec<Message>;
    fn send(&mut self, message: Message);
}

/// A trait for states that can respond to SystemRequest messages
trait SystemOp {
    fn request(&mut self, request: SystemRequest);
}

trait NameResolver {
    fn resolve_id_from_name(&self, name: &str) -> Option<AgentId>;
}

/// The state for an Alice or Bob Agent
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

/// A single unified behaviour that covers both Alice and Bob.
///
/// rust_agents only uses a single behaviour for all agents,
/// using an enum for these allows us to switch between the
/// two implementations for each agent.
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

/// An agent is a combination of its behaviour and its state.
///
/// In general the behaviour is stateless, does not change, and may be shared between more than one
/// agent, while the state is changed every timestep.
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

/// At the end of each timestep we get any system requests
/// for each acgent. To allow us to do this Agent must support
/// the AgentBase trait. (But since the available SystemRequests
/// might vary from simulation type to simulation type the
/// AgentBase is generic on the request type. - for Alice and Bob
/// the only simulation system level request is to remove them selves,
/// but in more complex systems it may involve spawning new agents)
impl AgentBase<SystemRequest> for Agent {
    fn empty_system_outbox(&mut self) -> Vec<SystemRequest> {
        self.state.system_outbox.drain(..).collect()
    }
}

/// For Alice and Bob we have simple maps form ids to agents,
/// and names to ids.
///
/// This is overly generic for a system which is only going to
/// be used for Alice and Bob, something like
///
/// struct GlobalContext {
///     alice: Option<Agent>
///     bob: Option<Agent>
/// }
///
/// would suffice.

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
