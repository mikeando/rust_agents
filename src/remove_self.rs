use crate::behaviour::Behaviour;
use crate::utils::{AgentId, BaseOp, SystemOp};

struct RemoveSelfBehaviour {}

#[derive(Clone, Debug)]
struct RemoveAgent(AgentId);

impl<STATE, CONTEXT, REQUEST> Behaviour<STATE, CONTEXT> for RemoveSelfBehaviour
where
    STATE: Clone + SystemOp<RequestType = REQUEST> + BaseOp,
    REQUEST: From<RemoveAgent>,
{
    fn act(&self, state: &STATE, _context: &CONTEXT) -> STATE {
        let mut state = state.clone();
        state.request(RemoveAgent(state.id()).into());
        state
    }
}
