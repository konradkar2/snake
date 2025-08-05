
use std::fmt::Debug;
use std::rc::Rc;
use serde::{Serialize, Deserialize};
pub trait FsmEvent: Debug {}
pub trait FsmState<E: FsmEvent>: Debug  {
    fn on_enter(self: &Self) {}
    fn on_exit(self: &Self) {}
    fn handle_event(self: &Self, event: &E) -> Option<Rc<dyn FsmState<E>>>;
}

#[derive(Debug)]
pub struct StateMachine<E: FsmEvent> {
    state: Rc<dyn FsmState<E>>,
}

impl<E: FsmEvent> StateMachine<E> {
    pub fn new(initial_state: Rc<dyn FsmState<E>>) -> Self {
        Self {
            state: initial_state,
        }
    }

    pub fn handle_event(&mut self, event: &E) {
        let new_state = self.state.handle_event(event);

        if let Some(new_state) = new_state {
            self.state.on_exit();
            self.state = new_state;
            self.state.on_enter();
        }
    }

    pub fn print_state(&self) {
        println!("Current state is {:?}", self.state);
    }
}