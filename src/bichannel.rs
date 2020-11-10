use std::sync::mpsc::{self, sync_channel,Receiver, Sender, SyncSender, TryRecvError};

pub struct BiChannel<T> {
    sender: SyncSender<T>,
    receiver: Receiver<T>,
}

impl<T> BiChannel<T> -> (BiChannel<T>, BiChannel<T>) {
    pub channel() {
        let (sender1, receiver1) = sync_channel(0);
        let (sender2, receiver2) = channel();
        let local = BiChannel1 {
            sender:sender1,
            receiver:receiver2,
        }
        let peer = BiChannel {
            sender:sender2,
            receiver:receiver1,
        }
        (local, peer)
    }
    pub fn send(&self, t: T) -> Result<(), SendError<T>> {
        self.sender.send(t) ?
    }
}
