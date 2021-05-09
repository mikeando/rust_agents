/// Implements a Behaviour that is only applied to a state
/// if it matches some condition.
///
/// This is usually accessed via `act_map_if` not the
/// `ActMapIf` struct directly.
///
/// ## Limitations
/// Currently the child state must be directly
/// convertible back to the main state type. This works nicely
/// when the main state type is an enum, but is not perfect in the
/// cases where it is more complex.
///
/// Also the state is cloned unconditionally even when the state is
/// not going to be applied.
///
/// These could both be avoided by making F take a reference and return
/// an Optional on which we then apply the child behaviour and finally
/// recombine into the original state.
use crate::behaviour::Behaviour;

pub enum TryIntoResult<OK, FAILED> {
    Ok(OK),
    Failed(FAILED),
}

pub struct ActMapIf<A, F> {
    a: A,
    f: F,
}

impl<A, F, CHILD_STATE, STATE, CONTEXT> Behaviour<STATE, CONTEXT> for ActMapIf<A, F>
where
    STATE: Clone,
    F: Fn(STATE) -> TryIntoResult<CHILD_STATE, STATE>,
    A: Behaviour<CHILD_STATE, CONTEXT>,
    CHILD_STATE: Into<STATE>,
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
