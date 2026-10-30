use std::rc::Rc;

pub type RowCallback<'a, Message> = Rc<dyn Fn(usize) -> Message + 'a>;
