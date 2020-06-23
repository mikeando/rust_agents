pub trait Behaviour<STATE, CONTEXT> {
    fn act(&self, state: STATE, context: &CONTEXT) -> STATE;
}
