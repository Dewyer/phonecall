use crate::{TelephoneCenter, TelephoneOperation};
use std::time::Duration;

pub struct Ping;

#[derive(Debug, Clone)]
pub struct PingParams {
    pub message: String,
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

#[tokio::test]
async fn basic_test_pam() {
    let center = TelephoneCenter::<SimpleCallCenter>::new();

    let phone_one = center.make_phone();
    let phone_two = phone_one.clone();

    let jt1 = {
        tokio::spawn(async move {
            let mut cc = 0;

            let mut center = center;
            loop {
                let next_call = center.handle_request().await.unwrap();

                match next_call {
                    SimpleTelephoneCall::Ping(params, resp) => {
                        resp.send(()).await.unwrap();

                        cc += 1;

                        match cc {
                            1 => {
                                assert_eq!(&params.message, "Hi!")
                            }
                            2 => {
                                assert_eq!(&params.message, "Hello bello!")
                            }
                            _ => {}
                        }

                        if cc == 2 {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        })
    };

    let jt2 = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let resp = phone_one
            .call::<Ping>(PingParams {
                message: "Hi!".to_string(),
            })
            .await
            .unwrap();

        dbg!(resp);
    });

    let jt3 = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;

        let resp = phone_two
            .call::<Ping>(PingParams {
                message: "Hello bello!".to_string(),
            })
            .await
            .unwrap();

        dbg!(resp);
    });

    let res = futures::future::join_all([jt1, jt2, jt3]).await;
    for re in res {
        // panic on any panics
        re.unwrap();
    }
}
