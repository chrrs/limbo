use tokio::sync::broadcast;

pub struct Shutdown {
    shutdown: bool,
    receiver: broadcast::Receiver<()>,
}

impl Shutdown {
    pub fn new(receiver: broadcast::Receiver<()>) -> Shutdown {
        Shutdown {
            shutdown: false,
            receiver,
        }
    }

    pub async fn recv(&mut self) {
        if self.shutdown {
            return;
        }

        let _ = self.receiver.recv().await;
        self.shutdown = true;
    }
}
