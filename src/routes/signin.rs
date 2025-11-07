use actix_web::{post ,web, HttpResponse, Responder};
use tokio::sync::oneshot;
use crate::{AppState, Request};
use serde::Deserialize;


#[derive(Deserialize)]
pub struct SigninPayload{
    username : String, // Each username has to be unique
    password : String
}

#[post("/signin")]
pub async fn signin (data : web::Data<AppState> , payload : web::Json<SigninPayload>) -> impl Responder {
    let (tx , mut rx) = oneshot::channel::<Result<String,String>>();
    let req = Request::Signin { 
        username: payload.username.clone(), 
        password: payload.password.clone(), 
        resp: tx 
    };
    if let Err(_) = data.worker.send(req).await {
        return HttpResponse::InternalServerError().body("Background worker creashed");
    }
    match rx.await {
        Ok(Ok(msg)) => HttpResponse::Ok().body(msg),
        Ok(Err(err)) => HttpResponse::BadRequest().body(err),
        Err(_) => HttpResponse::InternalServerError().body("No response from worker"),
    }
}