#![allow(unused_variables, unused_mut, unused_parens, dead_code)]
use crate::models::*;
use crate::utils::*;
use chrono::Utc;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

pub fn spawn_background_worker() -> mpsc::Sender<(Request)> {
    let (tx, mut rx) = mpsc::channel::<(Request)>(30);
    tokio::spawn(async move {
        let mut users: HashMap<String, User> = HashMap::new(); //  Hashmap of all users
        let mut markets: HashMap<String, Market> = HashMap::new();
        loop {
            match rx.recv().await {
                Some(req) => {
                    match req {
                        Request::Signup {
                            username,
                            password,
                            resp,
                        } => {
                            match users.get(&username) {
                                Some(user) => {
                                    let _ = resp.send(Err(
                                        "Username already exists , use a different username "
                                            .to_string(),
                                    ));
                                }
                                None => {
                                    // balance on signup is given = 5000
                                    users.insert(
                                        username.clone(),
                                        User {
                                            username: username.clone(),
                                            password,
                                            balance: 5000,
                                            holdings: HashMap::new(),
                                        },
                                    );
                                    let _ = resp.send(Ok(username));
                                }
                            }
                        }
                        Request::Signin {
                            username,
                            password,
                            resp,
                        } => {
                            match users.get(&username) {
                                Some(user) => {
                                    if verify_password(&password, &user.password) {
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
                        Request::CreateLimitOrder {
                            username,
                            stock_type,
                            price,
                            resp,
                            quantity,
                            ordertype,
                            market_id,
                        } => {
                            if let Some(user) = users.get_mut(&username) {
                                if let Some(market) = markets.get_mut(&market_id) {
                                    // check if market is not ended
                                    if market.is_settled {
                                        let _ = resp.send(Err(
                                            "Market is already settled. No new orders allowed."
                                                .to_string(),
                                        ));
                                        continue;
                                    }
                                    let mut order = Order {
                                        price,
                                        quantity,
                                        stock_type,
                                        username: username.clone(),
                                        timestamp: Utc::now(),
                                        ordertype,
                                        market_id: market_id.clone(),
                                    };
                                    let mut trades = market.add_limit_order(order, user);
                                    match trades {
                                        Ok(mut trades) => {
                                            for trade in trades.iter_mut() {
                                                let seller_name = &trade.from;
                                                let buyer_name = &trade.to;
                                                if let [Some(buyer), Some(seller)] = users
                                                    .get_disjoint_mut([buyer_name, seller_name])
                                                {
                                                    seller.balance +=
                                                        trade.trade_price * trade.trade_qty; //seller balance update after trade executed

                                                    let buyer_holdings = buyer
                                                        .holdings
                                                        .entry(market_id.clone())
                                                        .or_default();

                                                    match trade.stock_type {
                                                        StockType::StockA => {
                                                            buyer_holdings.stock_a +=
                                                                trade.trade_qty; //buyer's stock holdings update after trade executed
                                                        }
                                                        StockType::StockB => {
                                                            buyer_holdings.stock_b +=
                                                                trade.trade_qty; //buyer's stock update after trade executed
                                                        }
                                                    }
                                                    if username == trade.to {
                                                        let price_improvement =
                                                            price - trade.trade_price; // if user got stocks at better price than asked
                                                        if price_improvement > 0 {
                                                            let refund =
                                                                price_improvement * trade.trade_qty; // return the amount for that many stocks
                                                            buyer.balance += refund;
                                                        }
                                                    }
                                                }
                                            }
                                            let msg = if trades.is_empty() {
                                                "Order placed, waiting to be matched.".to_string()
                                            } else {
                                                format!("{:?}", trades)
                                            };
                                            let _ = resp.send(Ok(msg));
                                        }
                                        Err(err) => {
                                            let _ = resp.send(Err(err));
                                        }
                                    }
                                    //  balance update of both the parties done here
                                } else {
                                    let _ = resp.send(Err("Invalid option".to_string()));
                                }
                            } else {
                                let _ = resp.send(Err("User not found".to_string()));
                                continue;
                            }
                        }

                        Request::CreateMarketOrder {
                            username,
                            stock_type,
                            quantity,
                            ordertype,
                            resp,
                            market_id,
                        } => {
                            if let Some(user) = users.get_mut(&username) {
                                if let Some(market) = markets.get_mut(&market_id) {
                                    // checking if market has not ended
                                    if market.is_settled {
                                        let _ = resp.send(Err(
                                            "Market is already settled. No new orders allowed."
                                                .to_string(),
                                        ));
                                        continue;
                                    }
                                    let trades = market.execute_market_order(
                                        username.clone(),
                                        ordertype,
                                        quantity,
                                        stock_type,
                                        user,
                                        market_id.clone()
                                    );
                                    match trades {
                                        Ok(trades) => {
                                            for trade in trades.iter() {
                                                let buyer_name = &trade.to;
                                                let seller_name = &trade.from;
                                                // handling self-trade
                                                if seller_name == buyer_name {
                                                    continue;
                                                }
                                                if let [Some(buyer), Some(seller)] = users
                                                    .get_disjoint_mut([buyer_name, seller_name])
                                                {                               
                                                    seller.balance += trade.trade_price * trade.trade_qty;
                                                    let buyer_holdings = buyer
                                                        .holdings
                                                        .entry(market_id.clone())
                                                        .or_default();
                                            
                                                    match trade.stock_type {
                                                        StockType::StockA => {
                                                            buyer_holdings.stock_a += trade.trade_qty;
                                                            
                                                        }
                                                        StockType::StockB => {
                                                            buyer_holdings.stock_b += trade.trade_qty;
                                                            
                                                        }
                                                    }
                                                }
                                            }
                                            let msg = if trades.is_empty() {
                                                "Order placed, waiting to be matched.".to_string()
                                            } else {
                                                format!("{:?}", trades)
                                            };
                                            let _ = resp.send(Ok(msg));
                                        }
                                        Err(err) => {
                                            let _ = resp.send(Err(err));
                                        }
                                    }
                                } else {
                                    let _ = resp.send(Err("Invalid option".to_string()));
                                }
                            } else {
                                let _ = resp.send(Err("User not found".to_string()));
                                continue;
                            }
                        }
                        Request::CreateMarket {
                            username,
                            market_name,
                            resp,
                        } => {
                            if !users.contains_key(&username) {
                                let _ = resp.send(Err("User does not exist".to_string()));
                                continue;
                            }
                            let market = Market::initialise_market(market_name, username);
                            let market_id = market.market_id.clone();
                            match  markets.insert(market.market_id.to_string(), market) {
                                Some(market) => {
                                    let _ = resp.send(Err("Market already exists".to_string()));
                                }
                                None => {
                                    let _ = resp.send(Ok(market_id));   // hashmap insert returns none when added and Some when already exists
                                }
                            };
                            
                        }
                        Request::MergeStocks { 
                            username, 
                            market_id, 
                            amount, 
                            resp
                         } => {
                            if let Some(user) = users.get_mut(&username) {
                                // check holdings of both stock
                                let holdings = user.holdings.entry(market_id).or_default();
                                if holdings.stock_a < amount || holdings.stock_b < amount {
                                    let _ = resp.send(Err("Insufficient token pairs to redeem".to_string()));
                                    continue;
                                }

                                holdings.stock_a -= amount;
                                holdings.stock_b -= amount;
                                user.balance += amount;
                                let _ = resp.send(Ok(format!("Redeemed {} pairs for ${}", amount, amount)));
                            }
                         }
                         Request::SplitStocks { 
                            username, 
                            market_id, 
                            amount, 
                            resp
                         } => {
                            if let Some(user) = users.get_mut(&username){
                                // check collateral and then lock
                                if user.balance < amount {
                                    let _ = resp.send(Err("Insufficient funds to mint".to_string()));
                                    continue;
                                }
                                // checking if market exists
                                let market = markets.get(&market_id) ;
                                if market.is_none() {
                                    let _ = resp.send(Err("Market does not exists".to_string()));
                                    continue;
                                }
                                user.balance -= amount; // lock collateral
                                // mint equal amount a and b stocks to user
                                let holdings = user.holdings.entry(market_id.clone()).or_insert(
                                    UserHoldings { 
                                        stock_a:0 , 
                                        stock_b: 0
                                    }
                                );
                                holdings.stock_a += amount;
                                holdings.stock_b += amount;
                                let _ = resp.send(Ok(format!("Minted {} of Stock A and B", amount)));
                            }
                            else{
                                let _ = resp.send(Err("User not found".to_string()));
                            }
                         }
                         Request::UserDetails { 
                            username, 
                            resp 
                        } => {
                             let user = match users.get(&username) { 
                                Some(user) => {
                                    let userDetails = UserDetails{
                                        balance : user.balance,
                                        holdings : user.holdings.clone()
                                    };
                                    let _ = resp.send(Ok(userDetails));
                                }None => {
                                    let _ = resp.send(Err("User does not exist".to_string()));
                                    continue;
                                }
                             };
                        }
                        Request::GetOrderbook { market_id
                            , resp
                         } => {
                            let market = markets.get(&market_id) ;
                            if market.is_none() {
                                let _ = resp.send(Err("Market does not exists".to_string()));
                                continue;
                            }
                            let market = market.unwrap();
                            let _ = resp.send(Ok(Orderbooks { stock_a: market.stock_a.clone(), stock_b: market.stock_b.clone()}));
                         }
                    }
                }
                None => break,
            }
        }
    });
    tx
}
