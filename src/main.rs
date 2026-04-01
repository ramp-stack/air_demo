use air::names::{Secret, Signed, secp256k1::{Signed as KeySigned, SecretKey}, Resolver, Name, Error, Id};
use air::Request as ChandlerRequest;
use air::storage::{Request, Response};
use air::Purser;

use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel};
use tokio::time::{self, Duration};

mod tui;

const ROOM_KEY: &str = "\"4daa8e9afdecd8cc5e5fe59eea8c02887b59b3eb5219e247a89fbb1e52c1aff2\"";

async fn listen_to_room(key: SecretKey, sender: UnboundedSender<(String, String)>, mut receiver: UnboundedReceiver<Signed<String>>) -> Result<(), Error> {
    let mut interval = time::interval(Duration::from_millis(10));
    let mut index = 0;

    let mut outgoing: Option<Signed<String>> = None;

    loop {
        if outgoing.is_none() {outgoing = receiver.try_recv().ok();}
        let key = key.derive(&[index]);
        let request = outgoing.as_ref().map(|signed|
            Request::Create(KeySigned::new(&key, key.public_key().encrypt(serde_json::to_vec(&signed).unwrap())), true)
        ).unwrap_or(Request::Read(key.public_key(), true));

        match Purser.send(&mut Resolver, &Name::orange_me(), ChandlerRequest::Service(request)).await.unwrap().storage().unwrap() {
            Response::Private(_, p) => {
                index += 1;
                if let Some(signed) = key.decrypt(&p).ok().and_then(|p| serde_json::from_slice::<Signed<String>>(&p).ok()) {
                    match signed.verify(&mut Resolver, None, None).await.ok() {
                        Some(signer) => {
                            sender.send((signer.to_string(), signed.into_inner())).unwrap();
                        },
                        None => {
                            sender.send(("Signature Validator".to_string(), format!("Someone tried to impersonate '{}'", signed.signer()))).unwrap();
                        }
                    }
                }
            },
            Response::Receipt(m) if m.as_ref().hash == Id::MIN => {
                //Got an empty response you are at the top of the channel
                interval.tick().await;
            },
            Response::Receipt(_) => {
                index += 1;
                if let Some(signed) = outgoing.take() {sender.send((signed.signer().to_string(), signed.into_inner())).unwrap();}
                outgoing = receiver.try_recv().ok();
            }
            _ => Err(Error::InvalidMessage)?,
        }
    }
}

#[tokio::main]
async fn main() {
    let room_key: SecretKey = serde_json::from_str(ROOM_KEY).unwrap();
    let secret = Secret::new();

    let mut tui = tui::Tui::new().unwrap();
    let (sender, r) = unbounded_channel();
    let (s, mut receiver) = unbounded_channel();

    tokio::task::spawn(listen_to_room(room_key, s, r));

    loop {
        while let Ok((signer, msg)) = receiver.try_recv() {
            tui.add_message(signer, msg);
        }

        if let Some(message) = tui.tick().unwrap() {
            sender.send(Signed::new(&secret, message).unwrap()).unwrap();
        }
    }
}
