//! A tiny easy to use helper that makes function calls through threads/ tasks easier with tokio channels
//! For an example check `cargo run --example simple` at `examples/simple.rs`

use crate::error::Error;
use std::{fmt::Debug, marker::PhantomData};
use tokio::sync::mpsc;

#[macro_use]
pub mod macros;

pub mod error;

#[cfg(test)]
pub mod end_to_end_test;

/// Describes a telephone operation
/// An operation can be callable on multiple call centers
pub trait TelephoneOperation {
    type Parameters: Debug + Clone + Send + 'static;
    type ReturnValue: Debug + Send + 'static;
}

/// Usually implemented automatically, when done so, it signifies that an operation is callable
/// on the given call center
pub trait MakeCallOn<Call: CallCenter>: TelephoneOperation {
    const NAME: &'static str;
    fn make_call(request: Self::Parameters) -> (Call::CallEnum, mpsc::Receiver<Self::ReturnValue>);
}

/// Trait defining the call enum for a given call center
pub trait CallCenter: Clone + Debug {
    type CallEnum: Send + Sync + Debug + Clone + 'static;
}

/// The telephone center can be used to handle calls coming from callees (phones)
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

/// Created by a telephone center, it can be used to make calls to it, its cloneable
#[derive(Clone, Debug)]
pub struct Phone<Call: CallCenter> {
    tx: mpsc::Sender<Call::CallEnum>,
}

impl<Call: CallCenter> Phone<Call> {
    /// Does a blocking call to the call center
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

    /// Does a call to the call center and doesnt wait for the response
    pub async fn call_no_response<Operation>(
        &self,
        parameters: Operation::Parameters,
    ) -> Result<(), Error>
    where
        Operation: TelephoneOperation + MakeCallOn<Call>,
    {
        #[cfg(feature = "tracing")]
        tracing::trace!(
            "calling::{}({:?})",
            <Operation as MakeCallOn<Call>>::NAME,
            &parameters
        );

        let (req, _) = Operation::make_call(parameters);
        self.tx.send(req).await.map_err(|_| Error::Unknown)?;

        Ok(())
    }
}
