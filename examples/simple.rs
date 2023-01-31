use anyhow::bail;
use chrono::{NaiveDateTime, Utc};
use phonecall::{call_center, call_center_handlers, TelephoneCenter, TelephoneOperation};

// Create an identifier that identifies our operation
pub struct Ping;

// We can define a request object, pasically the parameters of the
#[derive(Debug, Clone)]
pub struct PingParams {
    pub sent_at: NaiveDateTime,
}

impl TelephoneOperation for Ping {
    type Parameters = PingParams;
    type ReturnValue = ();
}

pub struct HelloWorld;

impl TelephoneOperation for HelloWorld {
    type Parameters = ();
    type ReturnValue = Result<String, anyhow::Error>;
}

call_center!(SimpleCallCenter, SimpleTelephoneCall { Ping, HelloWorld });

async fn handle_ping(_: (), params: PingParams) {
    println!("ping sent at: {}", params.sent_at);
}

async fn handle_hello_world(_: (), _: ()) -> Result<String, anyhow::Error> {
    bail!("unimplemented!");
}

call_center_handlers!(
    handler,
    (),
    SimpleTelephoneCall
    {
        Ping => handle_ping,
        HelloWorld => handle_hello_world,
    }
);

#[tokio::main]
async fn main() {
    let mut center = TelephoneCenter::<SimpleCallCenter>::new();

    let phone_one = center.make_phone();

    let caller_handle = tokio::spawn(async move {
        phone_one
            .call::<Ping>(PingParams {
                sent_at: Utc::now().naive_utc(),
            })
            .await
            .unwrap();
    });

    let handler_handle = tokio::spawn(async move {
        let call = center.handle_request().await.unwrap();

        // This will print when the ping was sent from the other call
        handler((), call).await;
    });

    caller_handle.await.expect("called failed");
    handler_handle.await.expect("handler failed");
}
