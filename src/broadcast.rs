use crate::{CallCenter, MakeCallOn, Phone, TelephoneOperation};

pub struct Broadcast<Cc: CallCenter> {
    phones: Vec<Phone<Cc>>,
}

impl<Cc: CallCenter> Default for Broadcast<Cc> {
    fn default() -> Self {
        Broadcast::new()
    }
}

impl<Cc: CallCenter> Broadcast<Cc> {
    pub fn new() -> Self {
        Self { phones: vec![] }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.phones.is_empty()
    }

    fn clear_dead_phones(&mut self) {
        self.phones.retain(|ph| ph.is_alive())
    }

    pub fn attach_phone(&mut self, phone: Phone<Cc>) {
        self.phones.push(phone);

        self.clear_dead_phones();
    }

    /// Returns any number of responses from all attached and alive phones
    pub async fn call<Operation>(
        &self,
        parameters: Operation::Parameters,
    ) -> Vec<Operation::ReturnValue>
    where
        Operation: TelephoneOperation + MakeCallOn<Cc>,
    {
        futures::future::join_all(
            self.phones
                .iter()
                .map(|ph| ph.call::<Operation>(parameters.clone())),
        )
        .await
        .into_iter()
        .filter_map(|el| el.ok())
        .collect()
    }

    /// Does a non blocking call to all the attached and alive phones
    pub async fn call_no_response<Operation>(&self, parameters: Operation::Parameters)
    where
        Operation: TelephoneOperation + MakeCallOn<Cc>,
    {
        futures::future::join_all(
            self.phones
                .iter()
                .map(|ph| ph.call_no_response::<Operation>(parameters.clone())),
        )
        .await;
    }
}
