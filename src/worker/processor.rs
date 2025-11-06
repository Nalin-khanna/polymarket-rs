#![allow(unused_variables , unused_mut, unused_parens , dead_code)]
use tokio::sync::{mpsc, oneshot};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Request {
    Signup {
        username: String,
        password : String,
        resp: oneshot::Sender<Result<String, String>>
    },
    Signin {
        username: String,
        password : String,
        resp: oneshot::Sender<Result<String, String>>
    }
}

pub struct User {
    username : String ,
    password : String
}

pub fn spawn_background_worker () -> mpsc::Sender<(Request)>{
    let (tx , mut rx) = mpsc::channel::<(Request)>(30);
    tokio::spawn(async move {
        let mut users : HashMap<String, User> = HashMap::new();
        loop { 
            match rx.recv().await {
                Some(req) => {
                    match req {
                        Request::Signup { username, password,  resp } => {
                            
                            users.insert(username.clone(), User { username : username.clone(), password });
                            let _ = resp.send(Ok(username));
                        }
                        Request::Signin { username, password, resp } => {
                            match users.get(&username) {
                                Some(user) => {
                                    if user.password == password {
                                        println!("User signed in: {}", username); 
                                        // Send Ok with the username
                                        let _ = resp.send(Ok(username));
                                    } else {
                                        let _ = resp.send(Err("Invalid password".to_string()));
                                    }
                                }
                                None => {
                                    // User not found
                                    let _ = resp.send(Err("User not found".to_string()));
                                }
                            }
                        }
                    }
                }
                None => break
            }
        }
    });
    tx
}

