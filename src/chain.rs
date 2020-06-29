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
