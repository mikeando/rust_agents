use crate::behaviour::Behaviour;

pub enum TryIntoResult<OK, FAILED> {
    Ok(OK),
    Failed(FAILED),
}

pub trait TryInto<A> {
    fn try_into() -> TryIntoResult<A, Self>
    where
        Self: std::marker::Sized;
}

pub struct ActMapIf<A, F> {
    a: A,
    f: F,
}

impl<A, F, NEWSTATE, STATE, CONTEXT> Behaviour<STATE, CONTEXT> for ActMapIf<A, F>
where
    STATE: Clone,
    F: Fn(STATE) -> TryIntoResult<NEWSTATE, STATE>,
    A: Behaviour<NEWSTATE, CONTEXT>,
    NEWSTATE: Into<STATE>,
{
    fn act(&self, state: &STATE, context: &CONTEXT) -> STATE {
        let fs = (self.f)(state.clone());
        match fs {
            TryIntoResult::Ok(fs) => self.a.act(&fs, context).into(),
            TryIntoResult::Failed(state) => state,
        }
    }
}

pub fn act_map_if<A, F>(f: F, a: A) -> ActMapIf<A, F> {
    ActMapIf { a, f }
}

#[cfg(test)]
mod test {

    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    struct A(u32);

    #[derive(Clone, PartialEq, Debug)]
    struct B(u32);

    #[derive(Clone, PartialEq, Debug)]
    enum AB {
        A(A),
        B(B),
    }

    impl Into<AB> for A {
        fn into(self) -> AB {
            AB::A(self)
        }
    }

    impl AB {
        fn try_as_A(self) -> TryIntoResult<A, AB> {
            match self {
                AB::A(a) => TryIntoResult::Ok(a),
                AB::B(_) => TryIntoResult::Failed(self),
            }
        }
    }

    struct IncrementABehaviour {}

    impl Behaviour<A, ()> for IncrementABehaviour {
        fn act(&self, state: &A, _context: &()) -> A {
            A(state.0 + 1)
        }
    }

    #[test]
    fn test_map_if() {
        // Create a new behaviour that applied to the A variant of the AB enum
        // using an existing A only behaviour.
        let b = ActMapIf {
            a: IncrementABehaviour {},
            f: |x: AB| x.try_as_A(),
        };

        let _m: &dyn Behaviour<AB, ()> = &b;

        {
            let state = AB::A(A(10));
            let new_state = b.act(&state, &());
            assert_eq!(new_state, AB::A(A(11)));
        }

        {
            let state = AB::B(B(7));
            let new_state = b.act(&state, &());
            assert_eq!(new_state, AB::B(B(7)));
        }
    }

    #[test]
    fn test_act_map_if() {
        // Create a new behaviour that applied to the A variant of the AB enum
        let increment_a = IncrementABehaviour {};
        // using an existing A only behaviour.
        let b = act_map_if(|x: AB| x.try_as_A(), increment_a);

        let _m: &dyn Behaviour<AB, ()> = &b;

        {
            let state = AB::A(A(10));
            let new_state = b.act(&state, &());
            assert_eq!(new_state, AB::A(A(11)));
        }

        {
            let state = AB::B(B(7));
            let new_state = b.act(&state, &());
            assert_eq!(new_state, AB::B(B(7)));
        }
    }
}
