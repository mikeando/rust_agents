/// Test that checks that time is behaving as expected.
/// We set up several agents, each tracks the current time and compares
/// its time with its neighbours, ensuring that they are consistent.
use rust_agents::{
    behaviour::Behaviour,
    map_context::SimpleMapContext,
    utils::{step_agents, AgentId},
};

#[derive(Clone)]
struct TimeCheckAgent {
    id: AgentId,
    current_time: u32,
    detected_bad_time: bool,
}

impl TimeCheckAgent {
    pub fn new(id: u64) -> TimeCheckAgent {
        TimeCheckAgent {
            id: AgentId(id),
            current_time: 0,
            detected_bad_time: false,
        }
    }
}

type TimeCheckContext = SimpleMapContext<TimeCheckAgent>;

struct TimeCheckBehaviour;

/// Unlike most of the other behaviours we're not going to
/// try to make this extensible for other Agents and Contexts.
/// This makes things like accessing all the other nodes easier
/// (we dont need to set up a NeighbourhoodContext like in the Boids example)
impl Behaviour<TimeCheckAgent, TimeCheckContext> for TimeCheckBehaviour {
    fn act(&self, state: &TimeCheckAgent, context: &TimeCheckContext) -> TimeCheckAgent {
        let mut all_match = true;
        for n in context.agents.values() {
            if all_match && (n.current_time != state.current_time) {
                all_match = false;
            }
        }

        let mut new_state = state.clone();
        new_state.current_time += 1;
        new_state.detected_bad_time = new_state.detected_bad_time || !all_match;
        new_state
    }
}

#[test]
fn test_full_example() {
    let mut context = TimeCheckContext::new();
    context.agents = (0..10)
        .map(|i| (AgentId(i), TimeCheckAgent::new(i)))
        .collect();

    for _i in 0..10 {
        step_agents(&TimeCheckBehaviour, &mut context);
    }

    for (id, agent) in context.agents {
        assert!(
            !agent.detected_bad_time,
            "agent {} seen neighbours with an invalid time",
            id.0
        );
        assert_eq!(
            agent.current_time, 10,
            "agent {} has count {}",
            id.0, agent.current_time
        );
    }
}
