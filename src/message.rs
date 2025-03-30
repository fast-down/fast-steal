use crate::task::Tasks;

pub enum Message<Idx> {
    NewTask(Tasks<Idx>),
    UpdateTask(Idx),
    Stop,
}
