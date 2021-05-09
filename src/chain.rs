/// Creates a composite behaviour that applies the first behaviour and then
/// a second behaviour to the result of that first behaviour.
///
/// We don't use a Vec<Box<dyn Behaviour<STATE,CONTEXT>>> or similar as that
/// would incur a dynamic dispatch / virtual call overhead.
use crate::behaviour::Behaviour;

pub struct Chain<A, B> {
    a: A,
    b: B,
}

impl<A, B> Chain<A, B> {
    pub fn chain(a: A, b: B) -> Self {
        Chain { a, b }
    }
}

impl<STATE, CONTEXT, A, B> Behaviour<STATE, CONTEXT> for Chain<A, B>
where
    A: Behaviour<STATE, CONTEXT>,
    B: Behaviour<STATE, CONTEXT>,
{
    fn act(&self, state: &STATE, context: &CONTEXT) -> STATE {
        let temp_state = self.a.act(state, context);
        self.b.act(&temp_state, context)
    }
}
