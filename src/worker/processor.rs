#![allow(unused_variables , unused_mut, unused_parens , dead_code)]
use tokio::sync::mpsc;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Request {
    Signup {
        username: String,
        password : String,
        userid: u64,
        resp: oneshot::Sender<Result<String, String>>
    },
}

fn spawn_background_worker () -> mpsc::Sender<(Request)>{
    let (tx , mut rx) = mpsc::channel::<(Request)>(30);
    tokio::spawn(async move {
        loop { 
            match rx.recv().await {
                Some(req) => {
                    match req {
                        Request::Signup { username, password, userid, resp } => {
                            
                        }
                    }
                }
                None => break
            }
        }
        let mut users : HashMap<String, String> = HashMap::new();
    });
    tx
}