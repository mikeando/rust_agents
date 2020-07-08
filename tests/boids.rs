extern crate cgmath;
extern crate rand;
extern crate rust_agents;

use cgmath::prelude::*;
use cgmath::Vector3;
use rand::prelude::*;

use rust_agents::behaviour::Behaviour;
use rust_agents::chain::Chain;
use rust_agents::remove_self::RemoveSelfBehaviour;

use rust_agents::act_map_if::{act_map_if, TryIntoResult};
use rust_agents::utils::{
    map_tree_leaves, perform_system_actions, AgentBase, AgentId, BaseOp, System, SystemOp,
};

#[derive(Clone, Debug)]
struct CreateAgent {
    position: Vector3<f32>,
    direction: Vector3<f32>,
    rgb: (u8, u8, u8),
}

struct FlockCreator {}

impl FlockCreator {
    fn random_unit_vec(rand: &mut impl Rng) -> Vector3<f32> {
        Vector3::new(
            rand.gen::<f32>() - 0.5,
            rand.gen::<f32>() - 0.5,
            rand.gen::<f32>() - 0.5,
        )
        .normalize()
    }

    fn random_color(rand: &mut impl Rng) -> (u8, u8, u8) {
        (rand.gen(), rand.gen(), rand.gen())
    }
}

impl<STATE, CONTEXT, REQUEST> Behaviour<STATE, CONTEXT> for FlockCreator
where
    STATE: Clone + SystemOp<RequestType = REQUEST>,
    REQUEST: From<CreateAgent>,
{
    fn act(&self, state: &STATE, _context: &CONTEXT) -> STATE {
        let mut state = state.clone();

        let width: usize = 10;
        //     const { dimension, agent_count } = context.globals();
        // let dimension = 2;
        let agent_count = 10;
        // TODO: Get this from a mutable context!
        let mut rng = rand::thread_rng();

        for i in 0..agent_count {
            state.request(
                CreateAgent {
                    position: Vector3::new((i % width) as f32, (i / width) as f32, 1.0),
                    direction: FlockCreator::random_unit_vec(&mut rng),
                    rgb: FlockCreator::random_color(&mut rng),
                    // behaviors: ["flock.js"],
                }
                .into(),
            )
        }
        state
    }
}

#[derive(Clone)]
struct FlockGlobals {
    cohesion: f32,
    inertia: f32,
    alignment: f32,
    preferred_flock_size: f32,
}

trait PositionAndDirectionOp {
    fn position(&self) -> Vector3<f32>;
    fn direction(&self) -> Vector3<f32>;
    fn set_position(&mut self, p: Vector3<f32>);
    fn set_direction(&mut self, v: Vector3<f32>);
}

trait NeighborhoodContext<STATE> {
    fn for_each_neighbour<F>(&self, state: &STATE, f: F)
    where
        F: FnMut(&STATE, &STATE);
}

trait FlockGlobalContext {
    fn globals(&self) -> FlockGlobals;
}

struct FlockBehaviour {}

impl<STATE, CONTEXT> Behaviour<STATE, CONTEXT> for FlockBehaviour
where
    STATE: Clone + PositionAndDirectionOp,
    CONTEXT: FlockGlobalContext + NeighborhoodContext<STATE>,
{
    //   /**
    //    * This behaviour calculates the direction of a boid based
    //    * to rules of alignment, cohesion and separation.
    //    */
    fn act(&self, state: &STATE, context: &CONTEXT) -> STATE {
        let mut state = state.clone();
        let FlockGlobals {
            cohesion,
            inertia,
            alignment,
            preferred_flock_size,
        } = context.globals();

        let position = state.position();
        let direction = state.direction();

        // Alignment: steer along the average direction of neighbors
        let mut align_vec = direction.clone();
        let mut count = 1.0;
        context.for_each_neighbour(&state, |_state: &STATE, n: &STATE| {
            align_vec += n.direction();
            count += 1.0;
        });
        align_vec /= count;

        // Cohesion: steer to move towards the average position (center of mass) of neighbors
        // Separation: steer to avoid crowding local flockmates
        let mut avg_position = position.clone();
        context.for_each_neighbour(&state, |_state: &STATE, n: &STATE| {
            avg_position += n.position();
        });

        avg_position /= count;
        let cohesion_vec = avg_position - position;

        // Scale cohesion constant by difference from ideal flock size
        // This accounts for separation when the scaling is negative
        let scaled_cohesion = cohesion * (preferred_flock_size - count);

        // Calculate new direction
        let new_direction =
            (inertia * direction + scaled_cohesion * cohesion_vec + alignment * align_vec)
                .normalize();

        let new_position = position + new_direction;

        state.set_position(new_position);
        state.set_direction(new_direction);

        state
    }
}

#[derive(Clone, Debug)]
enum SystemRequest {
    CreateAgent(CreateAgent),
    RemoveAgent(RemoveAgent),
}

impl From<CreateAgent> for SystemRequest {
    fn from(request: CreateAgent) -> SystemRequest {
        SystemRequest::CreateAgent(request)
    }
}

impl From<RemoveAgent> for SystemRequest {
    fn from(request: RemoveAgent) -> SystemRequest {
        SystemRequest::RemoveAgent(request)
    }
}

#[derive(Clone, Debug)]
struct Creator {
    id: AgentId,
    system_outbox: Vec<SystemRequest>,
}

impl SystemOp for Creator {
    type RequestType = SystemRequest;
    fn request(&mut self, request: SystemRequest) {
        self.system_outbox.push(request);
    }
}

impl BaseOp for Creator {
    fn id(&self) -> AgentId {
        self.id
    }
}

#[derive(Clone, Debug)]
struct Boid {
    id: AgentId,
    position: Vector3<f32>,
    direction: Vector3<f32>,
    rgb: (u8, u8, u8),
}

impl PositionAndDirectionOp for Boid {
    fn position(&self) -> Vector3<f32> {
        self.position
    }
    fn direction(&self) -> Vector3<f32> {
        self.direction
    }
    fn set_position(&mut self, x: Vector3<f32>) {
        self.position = x;
    }
    fn set_direction(&mut self, v: Vector3<f32>) {
        self.direction = v;
    }
}

#[derive(Clone, Debug)]
enum Agent {
    Creator(Creator),
    Boid(Boid),
}

impl From<Creator> for Agent {
    fn from(v: Creator) -> Self {
        Agent::Creator(v)
    }
}

impl From<Boid> for Agent {
    fn from(v: Boid) -> Self {
        Agent::Boid(v)
    }
}

use std::collections::BTreeMap;

struct Context {
    globals: FlockGlobals,
    agents: BTreeMap<AgentId, Agent>,
    search_radius: f32,
}

impl Context {
    pub fn new() -> Context {
        Context {
            globals: FlockGlobals {
                alignment: 1.0,
                cohesion: 1.0,
                inertia: 1.0,
                preferred_flock_size: 4.0,
            },
            agents: BTreeMap::new(),
            search_radius: 10.0,
        }
    }
}

impl NeighborhoodContext<Boid> for Context {
    fn for_each_neighbour<F>(&self, self_boid: &Boid, mut f: F)
    where
        F: FnMut(&Boid, &Boid),
    {
        for (_k, v) in self.agents.iter() {
            match v {
                Agent::Boid(boid) => {
                    if self_boid.position.distance(boid.position) < self.search_radius {
                        f(self_boid, boid);
                    }
                }
                _ => {}
            }
        }
    }
}

impl FlockGlobalContext for Context {
    fn globals(&self) -> FlockGlobals {
        self.globals.clone()
    }
}

impl System<SystemRequest> for Context {
    type AgentType = Agent;
    fn apply_system_request(&mut self, action: SystemRequest) {
        match action {
            SystemRequest::CreateAgent(request) => {
                let next_id = self.agents.iter().map(|(k, _v)| k.0).max().unwrap() + 1;
                let new_agent = Agent::Boid(Boid {
                    id: AgentId(next_id),
                    direction: request.direction,
                    position: request.position,
                    rgb: request.rgb,
                });
                self.agents.insert(AgentId(next_id), new_agent);
            }
            SystemRequest::RemoveAgent(request) => {
                let agent_id = request.0;
                self.agents.remove(&agent_id);
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

impl Agent {
    pub fn try_into_creator(self) -> TryIntoResult<Creator, Agent> {
        match self {
            Agent::Creator(v) => TryIntoResult::Ok(v),
            _ => TryIntoResult::Failed(self),
        }
    }

    pub fn try_into_boid(self) -> TryIntoResult<Boid, Agent> {
        match self {
            Agent::Boid(v) => TryIntoResult::Ok(v),
            _ => TryIntoResult::Failed(self),
        }
    }
}

impl AgentBase<SystemRequest> for Agent {
    fn empty_system_outbox(&mut self) -> Vec<SystemRequest> {
        match self {
            Agent::Creator(state) => state.system_outbox.drain(..).collect(),
            _ => vec![],
        }
    }
}

#[test]
fn test_remove_self() {
    let remove_self = RemoveSelfBehaviour {};
    let _m: &dyn Behaviour<Creator, ()> = &remove_self;
}

#[test]
fn test_flock_creator() {
    let flock_creator = FlockCreator {};
    let _m: &dyn Behaviour<Creator, ()> = &flock_creator;
}

#[test]
fn test_create_and_remove() {
    let single_creator_behaviour = Chain::chain(FlockCreator {}, RemoveSelfBehaviour {});
    let _m: &dyn Behaviour<Creator, Context> = &single_creator_behaviour;
}

#[test]
fn test_create_and_remove_mapped() {
    let single_creator_behaviour = Chain::chain(FlockCreator {}, RemoveSelfBehaviour {});
    let mapped = act_map_if(
        |agent: Agent| agent.try_into_creator(),
        single_creator_behaviour,
    );
    let _m: &dyn Behaviour<Agent, Context> = &mapped;
}

#[test]
fn test_flock_behaviuor() {
    let flock_behaviour = FlockBehaviour {};
    let _m: &dyn Behaviour<Boid, Context> = &flock_behaviour;
}

#[test]
fn test_main() {
    let single_creator_behaviour = Chain::chain(FlockCreator {}, RemoveSelfBehaviour {});
    let flock_behaviour = FlockBehaviour {};

    let create_or_flock = Chain::chain(
        act_map_if(
            |agent: Agent| agent.try_into_creator(),
            single_creator_behaviour,
        ),
        act_map_if(|agent: Agent| agent.try_into_boid(), flock_behaviour),
    );

    let context = Context {
        globals: FlockGlobals {
            alignment: 1.0,
            cohesion: 1.0,
            inertia: 1.0,
            preferred_flock_size: 4.0,
        },
        agents: BTreeMap::new(),
        search_radius: 10.0,
    };

    let mut state = Agent::Creator(Creator {
        id: AgentId(0),
        system_outbox: vec![],
    });
    create_or_flock.act(&mut state, &context);

    let mut state = Agent::Boid(Boid {
        id: AgentId(1),
        position: Vector3::new(0.0, 0.0, 0.0),
        direction: Vector3::new(0.0, 0.0, 0.0),
        rgb: (1, 2, 3),
    });
    create_or_flock.act(&mut state, &context);
}

fn print_agents(context: &Context) {
    for (_agent_id, agent) in &context.agents {
        println!("  {:?}", agent)
    }
}

fn step_agents<B>(behaviour: &B, context: &mut Context)
where
    B: Behaviour<Agent, Context>,
{
    context.agents = map_tree_leaves(&context.agents, |agent| behaviour.act(agent, context));
}

#[test]
fn test_full_example() {
    let single_creator_behaviour = Chain::chain(FlockCreator {}, RemoveSelfBehaviour {});
    let flock_behaviour = FlockBehaviour {};

    let create_or_flock = Chain::chain(
        act_map_if(
            |agent: Agent| agent.try_into_creator(),
            single_creator_behaviour,
        ),
        act_map_if(|agent: Agent| agent.try_into_boid(), flock_behaviour),
    );

    let mut context = Context::new();
    context.agents.insert(
        AgentId(0),
        Agent::Creator(Creator {
            id: AgentId(0),
            system_outbox: vec![],
        }),
    );

    for i in 0..10 {
        println!("Step {}", i);
        print_agents(&context);
        step_agents(&create_or_flock, &mut context);

        // let messages = gather_messages(&mut context);
        perform_system_actions(&mut context);
        // deliver_messages(&mut context, messages);
    }
}
