use actix_web::{web, HttpResponse, Responder};
use tokio::sync::oneshot;
use crate::{AppState, Request};
use serde::Deserialize;
use crate::hash::*;

#[derive(Deserialize)]
pub struct SignupPayload{
    username : String,
    password : String
}
pub async fn signup(data : web::Data<AppState> , payload : web::Json<SignupPayload>) -> impl Responder {
    let (tx , mut rx) = oneshot::channel::<Result<String,String>>();
    let req = Request::Signup { 
        username: payload.username.clone(), 
        password: hash_password(&payload.password), 
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