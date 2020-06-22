
struct AliceState {}

impl AliceState {
    pub fn act(self, common:AgentCommon, outbox: &mut impl Outbox) -> (AgentCommon, AgentBody) {
        (common, self.into())
    }
}

impl Into<AgentBody> for AliceState {
    fn into(self) -> AgentBody { 
        AgentBody::Alice(self)
    }
}

struct BobState {}

#[derive(Debug)]
struct Greeting<'a> {
    msg:&'a str
}

trait Outbox {
    fn add_message(
        &mut self,
        to: &str,
        message_type: &str,
        body: MessageBody,
    );
}

#[derive(Debug)]
enum MessageBody<'a> {
    Greeting(Greeting<'a>)
}

impl BobState {
    pub fn act(self, common:AgentCommon, outbox: &mut impl Outbox) -> (AgentCommon, AgentBody) {
        outbox.add_message (
            "Alice",
            "greeting",
            MessageBody::Greeting(Greeting{
              msg: "Hello, Alice."
            })
        );
        (common, self.into())
    }
}

impl Into<AgentBody> for BobState {
    fn into(self) -> AgentBody { 
        AgentBody::Bob(self)
    }
}

enum AgentBody {
    Alice(AliceState),
    Bob(BobState),
}

impl AgentBody {
    pub fn act(self, common:AgentCommon, outbox: &mut impl Outbox) -> (AgentCommon, AgentBody) {
        match self {
            AgentBody::Alice(state) => state.act(common, outbox),
            AgentBody::Bob(state) => state.act(common, outbox),
        }
    }
}

#[derive(Debug)]
struct Message {}

struct AgentCommon {
    name: String,
    position: (i32, i32),
    inbox: Vec<Message>,
}

struct Agent {
    common: AgentCommon,
    body: AgentBody,
}

impl Agent {
    pub fn act(self, outbox: &mut impl Outbox) -> Agent {
        let mut agent = self;
        let (common, body): (AgentCommon, AgentBody) = agent.body.act(
            agent.common,
            outbox,
        );
        Agent{
            common,
            body
        }
    }
}

struct GlobalOutbox {}

impl Outbox for GlobalOutbox {
    fn add_message(
        &mut self,
        to: &str,
        message_type: &str,
        body: MessageBody,
    ) {
        println!("OUTBOX: Adding message from to:{} type:{}, body{:?}",
                to,
                message_type,
                body,
        )
    }
}

fn main() {

    let mut curstate:Vec<Agent> = vec![
        Agent{
            common:AgentCommon{ 
                name: "Alice".to_string(),
                position: (0,0),
                inbox: vec![],
            },
            body: AgentBody::Alice(AliceState{}),
        },
        Agent{
            common:AgentCommon{
            name: "Bob".to_string(),
            position: (3,0),
            inbox: vec![],
            },
            body:  AgentBody::Bob(BobState{}),
        }
    ];

    println!("Hello, world!");


    for i in 1..10 {
        let mut outbox = GlobalOutbox{};
        curstate = curstate.into_iter().map(|a| a.act(&mut outbox)).collect();
        println!("Step {}", i);
        for a in &curstate {
            println!("  {}  inbox={:?}", a.common.name, a.common.inbox)
        }
    }
}
