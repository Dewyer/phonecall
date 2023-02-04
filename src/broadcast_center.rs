use crate::{Broadcast, CallCenter, MakeCallOn, Phone, TelephoneOperation};
use std::{collections::HashMap, hash::Hash, sync::Arc};
use tokio::sync::RwLock;

pub struct BroadcastCenter<Cc: CallCenter, Top: PartialEq + Clone + Hash + Eq> {
    pub broadcasts: Arc<RwLock<HashMap<Top, Broadcast<Cc>>>>,
}

impl<Cc: CallCenter, Top: PartialEq + Clone + Hash + Eq> Default for BroadcastCenter<Cc, Top> {
    fn default() -> Self {
        BroadcastCenter::new()
    }
}

impl<Cc: CallCenter, Top: PartialEq + Clone + Hash + Eq> BroadcastCenter<Cc, Top> {
    pub fn new() -> Self {
        Self {
            broadcasts: Default::default(),
        }
    }

    /// Attaches a phone to the broadcast center for a specific topic
    pub async fn attach_to_broadcast(&self, topic: Top, phone: Phone<Cc>) {
        let mut broadcasts = self.broadcasts.write().await;
        let for_topic = broadcasts.entry(topic).or_insert(Default::default());

        for_topic.attach_phone(phone);
    }

    /// Calls all and any alive phones that are attached to a given topic
    pub async fn call_topic<Operation>(
        &self,
        topic: Top,
        parameters: Operation::Parameters,
    ) -> Vec<Operation::ReturnValue>
    where
        Operation: TelephoneOperation + MakeCallOn<Cc>,
    {
        let broadcasts = self.broadcasts.read().await;
        let for_topic = broadcasts.get(&topic);

        if let Some(for_topic) = for_topic {
            for_topic.call::<Operation>(parameters).await
        } else {
            vec![]
        }
    }

    /// Non blocking calls all and any alive phones that are attached to a given topic
    pub async fn call_topic_no_response<Operation>(
        &self,
        topic: Top,
        parameters: Operation::Parameters,
    ) where
        Operation: TelephoneOperation + MakeCallOn<Cc>,
    {
        let broadcasts = self.broadcasts.read().await;
        let for_topic = broadcasts.get(&topic);

        if let Some(for_topic) = for_topic {
            for_topic.call_no_response::<Operation>(parameters).await;
        }
    }
}
