use libp2p::gossipsub::{IdentTopic, TopicHash};

pub enum GossipTopic {
    Envelope,
    Tx,
    Block
}

impl GossipTopic {
    pub fn name(&self) -> String {
        match self {
            GossipTopic::Envelope => "/aprova/envelope".to_string(),
            GossipTopic::Tx => "/aprova/tx".to_string(),
            GossipTopic::Block => "/aprova/block".to_string(),
        }
    }

    pub fn ident(&self) -> IdentTopic {
        IdentTopic::new(self.name())
    }

    pub fn topic_hash(&self) -> TopicHash {
        self.ident().hash()
    }

    pub fn from_hash(hash: &TopicHash) -> Option<Self> {
        if hash == &GossipTopic::Tx.topic_hash() {
            Some(GossipTopic::Tx)
        } else if hash == &GossipTopic::Block.topic_hash() {
            Some(GossipTopic::Block)
        } else {
            None
        }
    }
}
