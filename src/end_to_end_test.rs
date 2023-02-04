use crate::{BroadcastCenter, TelephoneCenter, TelephoneOperation};
use lazy_static::lazy_static;
use std::time::Duration;
use tokio::time::timeout;

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
async fn basic_test_simple_cc() {
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

lazy_static! {
    static ref MAIN_BROADCAST_CENTER: BroadcastCenter<SimpleCallCenter, String> =
        { Default::default() };
}

#[tokio::test]
async fn basic_test_simple_broadcast_center() {
    let center_one = TelephoneCenter::<SimpleCallCenter>::new();
    let center_two = TelephoneCenter::<SimpleCallCenter>::new();

    MAIN_BROADCAST_CENTER
        .attach_to_broadcast("pie".to_string(), center_one.make_phone())
        .await;
    MAIN_BROADCAST_CENTER
        .attach_to_broadcast("pie".to_string(), center_two.make_phone())
        .await;

    let jt1 = {
        tokio::spawn(async move {
            let mut cc = 0;

            let mut center = center_one;
            loop {
                let next_call = center.handle_request().await.unwrap();

                match next_call {
                    SimpleTelephoneCall::Ping(_params, resp) => {
                        resp.send(()).await.ok();
                        cc += 1;
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
        let mut cc = 0;

        let mut center = center_two;
        loop {
            let next_call = center.handle_request().await.unwrap();

            match next_call {
                SimpleTelephoneCall::Ping(_params, resp) => {
                    resp.send(()).await.ok();
                    cc += 1;
                    if cc == 2 {
                        break;
                    }
                }
                _ => {}
            }
        }
    });

    let jt3 = tokio::spawn(async move {
        MAIN_BROADCAST_CENTER
            .call_topic::<Ping>(
                "pie".to_string(),
                PingParams {
                    message: "Hello!".to_string(),
                },
            )
            .await;

        MAIN_BROADCAST_CENTER
            .call_topic_no_response::<Ping>(
                "pie".to_string(),
                PingParams {
                    message: "Hello!".to_string(),
                },
            )
            .await;
    });

    let res = futures::join!(
        timeout(Duration::from_secs(1), jt1),
        timeout(Duration::from_secs(1), jt2),
        jt3,
    );

    res.0.unwrap().unwrap();
    res.1.unwrap().unwrap();
    res.2.unwrap();
}
