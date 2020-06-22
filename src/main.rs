use std::collections::BTreeMap;


#[derive(Debug, Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct AgentId(u64);

#[derive(Debug)]
struct AliceState {}

impl AliceState {
    pub fn act(self, common:AgentCommon, outbox: &mut impl Outbox) -> (AgentCommon, AgentBody) {
        let mut common = common;

        let greetings: Vec<&Message> = common.inbox.iter().filter(|m| {if let MessageBody::Greeting(_) = m.body  {true} else {false}} ).collect();
        if greetings.len() > 0 {
            common.color = Color::Blue;
        }

        (common, self.into())
    }
}

impl Into<AgentBody> for AliceState {
    fn into(self) -> AgentBody { 
        AgentBody::Alice(self)
    }
}


#[derive(Debug)]
struct BobState {}



trait Outbox {
    fn add_message(&mut self, message:Message);
}

trait Context {
    //TODO: This should be failable?
    fn resolve_id_from_name(&self, name:&str) -> AgentId;
}


impl BobState {
    pub fn act(self, common:AgentCommon, context:&impl Context, outbox: &mut impl Outbox) -> (AgentCommon, AgentBody) {
        let alice_id = context.resolve_id_from_name("Alice");
        let message = Message {
            from:common.id,
            to:alice_id,
            body:MessageBody::Greeting(Greeting{
                msg: "Hello, Alice.".to_string()
            })
        };
        outbox.add_message (message);
        (common, self.into())
    }
}

impl Into<AgentBody> for BobState {
    fn into(self) -> AgentBody { 
        AgentBody::Bob(self)
    }
}


#[derive(Debug)]
enum AgentBody {
    Alice(AliceState),
    Bob(BobState),
}

impl AgentBody {
    pub fn act(self, common:AgentCommon, context:&impl Context, outbox: &mut impl Outbox) -> (AgentCommon, AgentBody) {
        match self {
            AgentBody::Alice(state) => state.act(common, outbox),
            AgentBody::Bob(state) => state.act(common, context, outbox),
        }
    }
}

#[derive(Debug)]
struct Greeting {
    msg:String
}

#[derive(Debug)]
enum MessageBody {
    Greeting(Greeting)
}

#[derive(Debug)]
struct Message {
    to:AgentId,
    from:AgentId,
    body:MessageBody,
}

#[derive(Debug)]
enum Color {
    Black,
    Blue,
}

#[derive(Debug)]
struct AgentCommon {
    id: AgentId,
    name: String,
    position: (i32, i32),
    color: Color,
    inbox: Vec<Message>,
}

#[derive(Debug)]
struct Agent {
    common: AgentCommon,
    body: AgentBody,
}

impl Agent {
    pub fn act(self, context:&impl Context, outbox: &mut impl Outbox) -> Agent {
        let mut agent = self;
        let (common, body): (AgentCommon, AgentBody) = agent.body.act(
            agent.common,
            context,
            outbox,
        );
        Agent{
            common,
            body
        }
    }
}

struct GlobalOutbox {
    messages:Vec<Message>
}

impl Outbox for GlobalOutbox {
    fn add_message(&mut self, message:Message) {
        println!("OUTBOX: Adding message {:?}", message);
        self.messages.push(message);
    }
}

struct GlobalContext {}

impl Context for GlobalContext {
    fn resolve_id_from_name(&self, name: &str) -> AgentId {
        if name == "Alice" {
            return AgentId(111)
        } else if name == "Bob" {
            return AgentId(222);
        }
        panic!("Unknown name");
    }
}

fn main() {

    let mut curstate:Vec<Agent> = vec![
        Agent{
            common:AgentCommon{ 
                id:AgentId(111),
                name: "Alice".to_string(),
                position: (0,0),
                color:Color::Black,
                inbox: vec![],
            },
            body: AgentBody::Alice(AliceState{}),
        },
        Agent{
            common:AgentCommon{
                id:AgentId(222),
                name: "Bob".to_string(),
                position: (3,0),
                color:Color::Black,
                inbox: vec![],
            },
            body:  AgentBody::Bob(BobState{}),
        }
    ];

    let mut agents: BTreeMap<AgentId, Agent> = BTreeMap::new();
    let mut name_to_agent_id: BTreeMap<String, AgentId> = BTreeMap::new();
    
    for a in curstate {
        name_to_agent_id.insert(a.common.name.clone(), a.common.id);
        agents.insert(a.common.id, a);
    }

    println!("Hello, world!");

    let context = GlobalContext{};

    for i in 1..10 {
        let mut outbox = GlobalOutbox{messages:vec![]};
        agents = agents.into_iter().map(|(agent_id,agent)| (agent_id, agent.act(&context, &mut outbox))).collect();
        
        for (_agent_id, agent) in &mut agents {
            agent.common.inbox.clear()
        }

        for message in outbox.messages {
            agents.get_mut(&message.to).unwrap().common.inbox.push(message);
        }

        println!("Step {}", i);
        for (_agent_id, agent) in &agents {
            println!("  {:?}", agent)
        }
    }
}
