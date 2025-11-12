use std::collections::HashMap;

use crate::{UserHoldings, order::*};
use serde::Serialize;
use tokio::sync::oneshot;

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
    },
    CreateLimitOrder{
        username : String,
        stock_type : StockType , // Option A or Option B (yes or no)
        price : u64,
        quantity : u64,
        ordertype : Ordertype,
        market_id : String, 
        resp: oneshot::Sender<Result<String, String>>
    },
    CreateMarketOrder {
    username: String,
    stock_type: StockType,
    quantity: u64,
    ordertype: Ordertype,
    market_id : String,
    resp: oneshot::Sender<Result<String, String>>,
    },
    CreateMarket{
        username : String,
        market_name : String,
        resp: oneshot::Sender<Result<String, String>>
    },
    SplitStocks {
        username: String,
        market_id: String,
        amount: u64, // Amount of user balance to lock up
        resp: oneshot::Sender<Result<String, String>>,
    },
    MergeStocks {
        username: String,
        market_id: String,
        amount: u64, // Amount of YES+NO pairs to burn
        resp: oneshot::Sender<Result<String, String>>,
    },
    UserDetails{
        username : String ,
        resp: oneshot::Sender<Result<UserDetails, String>>,
    },
    GetOrderbook{
        market_id : String,
        resp: oneshot::Sender<Result<Orderbooks, String>>,
    }
}

#[derive(Debug , Clone , Serialize)]
pub struct UserDetails{
    pub balance : u64,
    pub holdings : HashMap<String ,UserHoldings > 
}

#[derive(Debug , Serialize )]
pub struct Orderbooks{
    pub stock_a : OrderBook,
    pub stock_b : OrderBook
}