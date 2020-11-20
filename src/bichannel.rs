use std::sync::mpsc::{
    self, channel, sync_channel, Receiver, RecvError, SendError, Sender, SyncSender,
};

pub struct BiChannel<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> BiChannel<T> {
    pub fn channel() -> (BiChannel<T>, BiChannel<T>) {
        let (sender1, receiver1) = channel();
        let (sender2, receiver2) = channel();
        let local = BiChannel {
            sender: sender1,
            receiver: receiver2,
        };

        let peer = BiChannel {
            sender: sender2,
            receiver: receiver1,
        };

        (local, peer)
    }
    pub fn send(&self, t: T) -> Result<(), SendError<T>> {
        self.sender.send(t)?
    }
    pub fn recv(&self) -> Result<T, RecvError> {
        self.receiver.recv()?
    }
}
