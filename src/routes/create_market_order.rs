use actix_web::{post ,web, HttpResponse, Responder};
use tokio::sync::oneshot;
use crate::{AppState, Request};
use serde::Deserialize;
use crate::order::*;

#[derive(Deserialize)]
struct MarketOrderPayload {
    username : String,
    option : Option , // Option A or Option B (yes or no)
    price : u64,
    quantity : u64,
    ordertype : Ordertype,
}

#[post("/marketorder")]
pub async fn create_market_order(data : web::Data<AppState> , payload : web::Json<MarketOrderPayload>) -> impl Responder {
    let (tx , mut rx) = oneshot::channel::<Result<String,String>>();
    let req = Request::CreateMarketOrder { 
        username: payload.username.clone(), 
        option: payload.option.clone(), 
        quantity:payload.quantity,
        ordertype: payload.ordertype.clone(), 
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