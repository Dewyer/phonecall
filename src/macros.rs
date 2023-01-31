#[macro_export]
macro_rules! call_center {
    ($call_center:ident, $enum_name:ident { $($body:ident),*$(,)* }) => {
        #[derive(Debug, Clone)]
        pub struct $call_center;

        $crate::as_item! {
            #[derive(Debug, Clone)]
            pub enum $enum_name { $($body(<$body as TelephoneOperation>::Parameters,tokio::sync::mpsc::Sender<<$body as TelephoneOperation>::ReturnValue>),)* }
        }

        $(impl $crate::MakeCallOn<$call_center> for $body {
            const NAME: &'static str = stringify!($enum_name::$body);

            fn make_call(request: Self::Parameters) -> ($enum_name, tokio::sync::mpsc::Receiver<Self::ReturnValue>) {
                let (tx, rx) = tokio::sync::mpsc::channel(1);

                (
                    $enum_name::$body(request, tx),
                    rx
                )
            }
        })*

        impl $crate::CallCenter for $call_center {
            type CallEnum = $enum_name;
        }

    };
}

#[macro_export]
macro_rules! call_center_handlers {
    ($handler_name:ident, $ctx_name:tt, $enum_name:ident { $($operation:ident => $handler_fn:tt),*$(,)* }) => {
        pub async fn $handler_name(ctx: $ctx_name, call: $enum_name) {
            match call {
                $($enum_name::$operation(req, res) => {
                    let result = $handler_fn(ctx, req).await;
                    res.send(result).await.unwrap();
                })*
            }
        }
    };
}

#[macro_export]
macro_rules! as_item {
    ($i:item) => {
        $i
    };
}
