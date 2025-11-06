#![allow(unused_variables , unused_mut, unused_parens , dead_code)]
use tokio::sync::{mpsc, oneshot};
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

pub struct User {
    username : String ,
    password : String
}

pub fn spawn_background_worker () -> mpsc::Sender<(Request)>{
    let (tx , mut rx) = mpsc::channel::<(Request)>(30);
    tokio::spawn(async move {
        let mut users : HashMap<u64, User> = HashMap::new();
        loop { 
            match rx.recv().await {
                Some(req) => {
                    match req {
                        Request::Signup { username, password, userid, resp } => {
                            let username_clone = username.clone();
                            users.insert(userid, User { username : username_clone, password });
                            let _ = resp.send(Ok(username));
                        }
                    }
                }
                None => break
            }
        }
    });
    tx
}

