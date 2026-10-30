use super::Step;

#[derive(Debug, Clone)]
pub enum BootEvent{
    Step(Step),
    Detail(String),
    Progress(f32),
    Failed(String),
    Ready,
}