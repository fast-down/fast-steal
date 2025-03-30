use crate::{message::Message, task::Tasks};
use crossbeam_channel::Sender;

pub struct Worker<Idx> {
    pub tx_task: Sender<Message<Idx>>,
    pub remain: Idx,
    pub tasks: Tasks<Idx>,
}
