use actix_web::{post ,web, HttpResponse, Responder};
use tokio::sync::oneshot;
use crate::{AppState, Orderbooks, Request, auth_extractor::AuthenticatedUser};
use serde::Deserialize;
use crate::order::*;

#[derive(Deserialize)]
struct GetOrderbookPayload {
    market_id : String
}

#[post("/get_orderbook")]
pub async fn get_ordrebook(data : web::Data<AppState> , payload : web::Json<GetOrderbookPayload> , username : AuthenticatedUser) -> impl Responder {
    let (tx , mut rx) = oneshot::channel::<Result<Orderbooks,String>>();
    let req = Request::GetOrderbook { 
        market_id: payload.market_id.to_string(), 
        resp: tx
    } ;
    if let Err(_) = data.worker.send(req).await {
        return HttpResponse::InternalServerError().body("Background worker creashed");
    }
    match rx.await {
        Ok(Ok(msg)) => HttpResponse::Ok().json(msg),
        Ok(Err(err)) => HttpResponse::BadRequest().body(err),
        Err(_) => HttpResponse::InternalServerError().body("No response from worker"),
    }
}