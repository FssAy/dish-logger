use std::prelude::v1::Option::Some;
pub use std::sync::Arc;
pub use tokio::sync::{oneshot, mpsc};


type Responder<T> = oneshot::Sender<T>;

#[derive(Debug)]
pub enum Directive {
    SendToken {
        token: String,
    },
    TakeToken {
        resp: Responder<Option<String>>,
    },
    CheckToken {
        resp: Responder<bool>,
    },
    Close,
}

pub async fn handler(mut receiver: mpsc::Receiver<Directive>) {
    let mut s_token: Option<String> = None;

    while let Some(directive) = receiver.recv().await {
        use Directive::*;

        match directive {
            SendToken { token } => {
                s_token = Some(token);
            }
            TakeToken { resp } => {
                resp.send(std::mem::take(&mut s_token)).unwrap();
            }
            CheckToken { resp } => {
                resp.send(s_token.is_some()).unwrap();
            }
            Close => {
                break;
            }
        }
    }

    receiver.close();
}
