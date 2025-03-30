use crate::task::Tasks;
use crossbeam_channel::Sender;

pub struct Worker<Idx> {
    pub tx_task: Sender<Tasks<Idx>>,
    pub remain: Idx,
    pub tasks: Tasks<Idx>,
}
