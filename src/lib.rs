use crate::error::Error;
use std::{fmt::Debug, marker::PhantomData};
use tokio::sync::mpsc;

#[macro_use]
pub mod macros;

pub mod error;

#[cfg(test)]
pub mod end_to_end_test;

pub trait TelephoneOperation {
    type Parameters: Debug + Clone + Send + 'static;
    type ReturnValue: Debug + Send + 'static;
}

pub trait MakeCallOn<Call: CallCenter>: TelephoneOperation {
    const NAME: &'static str;
    fn make_call(request: Self::Parameters) -> (Call::CallEnum, mpsc::Receiver<Self::ReturnValue>);
}

pub trait CallCenter: Clone + Debug {
    type CallEnum: Send + Sync + Debug + Clone + 'static;
}

pub struct TelephoneCenter<Call: CallCenter> {
    cl: PhantomData<Call>,

    tx: mpsc::Sender<Call::CallEnum>,
    rx: mpsc::Receiver<Call::CallEnum>,
}

impl<Call: CallCenter> TelephoneCenter<Call> {
    pub fn new() -> TelephoneCenter<Call> {
        let (tx, rx) = mpsc::channel(100);

        Self {
            cl: Default::default(),
            tx,
            rx,
        }
    }

    pub fn make_phone(&self) -> Phone<Call> {
        Phone {
            tx: self.tx.clone(),
        }
    }

    pub async fn handle_request(&mut self) -> Result<Call::CallEnum, Error> {
        self.rx.recv().await.ok_or(Error::Unknown)
    }
}

impl<Call: CallCenter> Default for TelephoneCenter<Call> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct Phone<Call: CallCenter> {
    tx: mpsc::Sender<Call::CallEnum>,
}

impl<Call: CallCenter> Phone<Call> {
    pub async fn call<Operation>(
        &self,
        parameters: Operation::Parameters,
    ) -> Result<Operation::ReturnValue, Error>
    where
        Operation: TelephoneOperation + MakeCallOn<Call>,
    {
        #[cfg(feature = "tracing")]
        tracing::trace!(
            "calling::{}({:?})",
            <Operation as MakeCallOn<Call>>::NAME,
            &parameters
        );

        let (req, mut recv) = Operation::make_call(parameters);
        self.tx.send(req).await.map_err(|_| Error::Unknown)?;

        let resp = recv.recv().await.ok_or(Error::Unknown)?;

        #[cfg(feature = "tracing")]
        tracing::trace!("response on ::{}({:?})", Operation::NAME, &resp);
        Ok(resp)
    }
}
