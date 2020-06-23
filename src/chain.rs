
use crate::behaviour::Behaviour;

struct Chain<A,B> {
    a:A, 
    b:B,
}

impl <STATE,CONTEXT,A,B> Behaviour<STATE,CONTEXT> for Chain<A,B>
where
    A: Behaviour<STATE,CONTEXT>,
    B: Behaviour<STATE,CONTEXT>,
{
    fn act(&self, state:STATE, context:&CONTEXT) -> STATE {
        let temp_state = self.a.act(state, context);
        self.b.act(temp_state, context)
    }
}