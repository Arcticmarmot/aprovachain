use libp2p::gossipsub::IdentTopic;

pub enum GossipTopic {
    Tx,
    Block
}

impl GossipTopic {
    fn name(&self) -> String {
        match self {
            GossipTopic::Tx => "/aprova/tx".to_string(),
            GossipTopic::Block => "/aprova/block".to_string()
        }
    }

    fn ident(&self) -> IdentTopic {
        IdentTopic::new(self.name())
    }
}
