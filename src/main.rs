use std::collections::BTreeMap;


trait Behaviour<STATE,CONTEXT> {
    fn act(&self, state:STATE, context:&CONTEXT) -> STATE;
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct AgentId(u64);

trait BaseOp {
    fn id(&self) -> AgentId;
}

#[derive(Debug)]
enum Color {
    Black,
    Blue,
    Red,
}

trait ColorOp {
    fn get_color(&self) -> &Color;
    fn set_color(&mut self, color:Color);
}

trait MessageOp {
    // TODO: Better to use an iterator than allocate?
    // The Message type should be only those that Alice can understand
    fn inbox(&self) -> Vec<Message>;
    fn send(&mut self, message:Message) ;
}

trait SystemOp {
    fn request(&mut self, request:SystemRequest);
}

trait NameResolver {
    fn resolve_id_from_name(&self, name:&str) -> Option<AgentId>;
}

#[derive(Debug)]
struct AliceBehaviour {}

impl <STATE, CONTEXT> Behaviour<STATE,CONTEXT> for AliceBehaviour 
where 
    STATE:BaseOp + ColorOp + MessageOp + SystemOp 
{
    fn act(&self, state:STATE, _context:&CONTEXT) -> STATE {
        let mut state = state;

        let inbox = state.inbox();
        let greetings: Vec<&Message> = inbox
        .iter()
        .filter(|m| {
            if let MessageBody::Greeting(_) = m.body {
                true
            } else {
                false
            }
        })
        .collect();

        if greetings.len() > 0 {
            state.set_color(Color::Blue);
            state.request(SystemRequest{
                from:state.id(),
                body: SystemRequestBody::RemoveAgent(state.id())
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

#[derive(Debug)]
struct BobBehaviour {}

impl <STATE, CONTEXT> Behaviour<STATE, CONTEXT> for BobBehaviour 
where
    STATE:BaseOp + ColorOp + MessageOp + SystemOp,
    CONTEXT: NameResolver,
{
    fn act(&self, state:STATE, context:&CONTEXT) -> STATE {
        let mut state = state;
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
            },
            None => {
                state.request(SystemRequest{
                    from:state.id(),
                    body: SystemRequestBody::RemoveAgent(state.id()),
                });
            }
        };

        let inbox = state.inbox();
        let greetings: Vec<&Message> = inbox
            .iter()
            .filter(|m| {
                if let MessageBody::Greeting(_) = m.body {
                    true
                } else {
                    false
                }
            })
            .collect();

        if greetings.len() > 0 {
            state.set_color(Color::Red);
        }

        state
    }
}

#[derive(Debug)]
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
    fn set_color(&mut self, color:Color) {
        self.color = color
    }
}

impl MessageOp for AgentState {
    fn inbox(&self) -> Vec<Message> {
        self.inbox.clone()
    }
    fn send(&mut self, message:Message)  {
        self.outbox.push(message)
    }
}

impl SystemOp for AgentState {
    fn request(&mut self, request:SystemRequest) {
        self.system_outbox.push(request);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Greeting {
    msg: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RemoveAgentMessage {
    to_remove:AgentId
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MessageBody {
    Greeting(Greeting),
    RemoveAgentMessage(RemoveAgentMessage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Message {
    to: AgentId,
    from: AgentId,
    body: MessageBody,
}

#[derive(Debug)]
enum SystemRequestBody {
    RemoveAgent(AgentId)
}

#[derive(Debug)]
struct SystemRequest {
    from: AgentId,
    body: SystemRequestBody,
}


#[derive(Debug)]
enum AgentBehaviour {
    Alice(AliceBehaviour),
    Bob(BobBehaviour),
}



impl <STATE,CONTEXT> Behaviour<STATE, CONTEXT> for AgentBehaviour 
where
    STATE: BaseOp + ColorOp + MessageOp + SystemOp,
    CONTEXT: NameResolver,
{
    fn act(&self, state:STATE, context:&CONTEXT) -> STATE {
        match self {
            AgentBehaviour::Alice(behaviour) => behaviour.act(state,context),
            AgentBehaviour::Bob(behaviour) => behaviour.act(state,context),
        }
    }
}

#[derive(Debug)]
struct Agent {
    behaviour:AgentBehaviour,
    state:AgentState,
}

impl Agent {
    fn  act<CONTEXT>(self, context:&CONTEXT) -> Self 
    where CONTEXT: NameResolver
    {
        let Agent{behaviour, state} = self;
        let state = behaviour.act(state, context);
        Agent{
            behaviour,
            state,
        }
    }
}

struct GlobalContext {
    // let mut agents: BTreeMap<AgentId, Agent> = BTreeMap::new();
    name_to_agent_id: BTreeMap<String, AgentId>
}

impl GlobalContext {
    fn new() -> Self {
        GlobalContext { 
            name_to_agent_id: BTreeMap::new()
        }
    }
}

impl NameResolver for GlobalContext {
    fn resolve_id_from_name(&self, name:&str) -> Option<AgentId> {
        self.name_to_agent_id.get(name).cloned()
    }
}

fn main() {
    let curstate: Vec<Agent> = vec![
        Agent {
            behaviour: AgentBehaviour::Alice(AliceBehaviour{}),
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
            behaviour: AgentBehaviour::Bob(BobBehaviour{}),
            state: AgentState {
                id: AgentId(222),
                name: "Bob".to_string(),
                position: Some((3, 0)),
                color: Color::Black,
                inbox: vec![],
                outbox: vec![],
                system_outbox: vec![],
            }
        }
    ];

    let mut agents: BTreeMap<AgentId, Agent> = BTreeMap::new();
    let mut name_to_agent_id: BTreeMap<String, AgentId> = BTreeMap::new();

    for a in curstate {
        name_to_agent_id.insert(a.state.name.clone(), a.state.id);
        agents.insert(a.state.id, a);
    }

    let mut context = GlobalContext::new();
    context.name_to_agent_id = name_to_agent_id;

    for i in 0..10 {

        println!("Step {}", i);
        for (_agent_id, agent) in &agents {
            println!("  {:?}", agent)
        }

        agents = agents
            .into_iter()
            .map(|(agent_id, agent)| (agent_id, agent.act(&context)))
            .collect();

        let mut system_actions = vec![];
        let mut messages = vec![];
        for (_agent_id, agent) in &mut agents {
            agent.state.inbox.clear();
            messages.append(&mut agent.state.outbox);
            system_actions.append(&mut agent.state.system_outbox);
        }

        for action in system_actions {
            println!("ACTION: {:?}", action);
            match action.body {
                SystemRequestBody::RemoveAgent(agent_id) => {
                    println!("{:?} requested removal of {:?}", action.from, agent_id);
                    let agent = agents.remove(&agent_id).unwrap();
                    context.name_to_agent_id.remove(&agent.state.name);
                }
            }
        }

        for message in messages {
            println!("MESSAGE: {:?}", message);
            match agents.get_mut(&message.to) {
                Some(agent) => { agent.state.inbox.push(message); },
                None => { println!("No agent {:?}", message.to); }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    struct TestAgentData {
        id:AgentId,
        color:Color,
        inbox: Vec<Message>,
        outbox: Vec<Message>,
    }

    impl TestAgentData {
        pub fn new() -> Self {
            TestAgentData {
                id:AgentId(111),
                color:Color::Black,
                inbox:vec![],
                outbox:vec![],
            }
        }
    }


    impl BaseOp for TestAgentData {
        fn id(&self) -> AgentId {
            self.id
        }
    }

    impl ColorOp for TestAgentData {
        fn get_color(&self) -> &Color {
            &self.color
        }
        fn set_color(&mut self, color:Color) {
            self.color = color;
        }
    }

    impl MessageOp for TestAgentData {
        fn inbox(&self) -> Vec<Message> {
            self.inbox.clone()
        }
        fn send(&mut self, message:Message) {
            self.outbox.push(message);
        }
    }

    impl SystemOp for TestAgentData {
        fn request(&mut self, request:SystemRequest) {
            todo!()
        }
    }

    struct TestContext {}

    impl NameResolver for TestContext {
        fn resolve_id_from_name(&self, name:&str) -> Option<AgentId> {
            Some(AgentId(111))
        }
    }


    #[test]
    pub fn test_alice_unchanging() {

        let alice_state = TestAgentData::new();
        let context = TestContext{};
        let alice_behaviour = AliceBehaviour{};

        {
            let _t: &dyn Behaviour<TestAgentData, TestContext> = &alice_behaviour;
        }

        let alice_state = alice_behaviour.act(alice_state, &context);
    
        // Should have done nothing
        assert_eq!(alice_state.inbox, vec![]);
        assert_eq!(alice_state.outbox, vec![]);
    }

    #[test]
    pub fn test_bob_messages_alice() {

        let bob_state = TestAgentData::new();
        let context = TestContext{};
        let bob_behaviour = BobBehaviour{};

        {
            let _t: &dyn Behaviour<TestAgentData, TestContext> = &bob_behaviour;
        }

        let bob_state = bob_behaviour.act(bob_state, &context);
    
        // Should have done nothing
        assert_eq!(bob_state.inbox, vec![]);
        assert_eq!(bob_state.outbox.len(), 1);
    }
}
