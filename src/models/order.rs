
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{ BTreeMap , VecDeque};

use crate::User;

#[derive(Debug , Clone  , Serialize )]
pub struct Order {
   pub price : u64, 
   pub quantity : u64,
   pub stock_type: StockType,
   pub username : String,
   pub timestamp: DateTime<Utc>,
   pub ordertype: Ordertype,
   pub market_id : String
}
#[derive(Debug , Clone )]
pub struct Trade {
    pub from : String,  // always the user who's stocks are sold (seller)
    pub to : String ,   // always the user who buys the stocks (buyer)
    pub trade_qty : u64 ,
    pub trade_price : u64 ,
    pub stock_type : StockType
}
#[derive(Debug , Clone , PartialEq, Hash, Eq , Deserialize , Serialize)]
pub enum StockType {
    StockA,
    StockB 
}
#[derive(Debug , Clone , Deserialize , Serialize)]
pub enum Ordertype{
    Buy,
    Sell
}
#[derive(Debug, Serialize , Clone)]
pub struct OrderBook {
    pub buy: BTreeMap<u64, VecDeque<Order>>,
    pub sell: BTreeMap<u64, VecDeque<Order>>,
}
impl OrderBook {
    pub fn new () -> Self{
        Self { buy: BTreeMap::new(), sell: BTreeMap::new() }
    }
    pub fn add_limit_order(
        &mut self,
        mut order : Order,
        user : &mut User
    )-> Result<Vec<Trade> , String> {
        let mut trades = vec![];
        match order.ordertype {
            Ordertype::Buy => {
                let required_balance = order.price * order.quantity;
                if user.balance < order.price * order.quantity {
                    return Err(format!("Insufficient funds. Required: {}, Available: {}", required_balance, user.balance));
                }
                user.balance -= order.price * order.quantity; // Funds locked immediately

                while let Some((&lowest_sell_price , queue)) = self.sell.iter_mut().next(){
                    if order.price >= lowest_sell_price && order.quantity > 0{
                        if let Some (mut sell_order) = queue.pop_front(){
                            let trade_qty = order.quantity.min(sell_order.quantity);  // minimum quantity out of buy order and sell order popped from queue
                            order.quantity -= trade_qty;            //  minimum qty can only be matched
                            sell_order.quantity -= trade_qty;

                            trades.push(Trade{
                                from : sell_order.username.clone(),
                                to : order.username.clone(),
                                trade_qty,
                                trade_price : lowest_sell_price,
                                stock_type : sell_order.stock_type.clone()
                            });
                            if sell_order.quantity > 0 {          // if there is still qty left for sellorder, push it back to front of queue
                                queue.push_front(sell_order);
                            }
                        }else{
                            self.sell.remove(&lowest_sell_price);
                        }
                    }else{
                        break
                    }
                }
                if order.quantity > 0 {
                    self.buy.entry(order.price).or_insert_with(VecDeque::new).push_back(order);  // after the loop , if buy order has qty left , push it to buy BTREE
                }
            }
            Ordertype::Sell => {
                let holdings = user.holdings.entry(order.market_id.clone()).or_default();
                let available_stock = match order.stock_type {
                        StockType::StockA => &mut holdings.stock_a,
                        StockType::StockB => &mut holdings.stock_b,
                };
                if *available_stock < order.quantity {
                        return Err(format!("Insufficient stock. Required: {}, Available: {}", order.quantity, available_stock));
                }
                *available_stock -= order.quantity;  // lock the users stock 
                while let Some((&highest_buy_price, queue)) = self.buy.iter_mut().next_back() {
                    if order.price <= highest_buy_price && order.quantity > 0 {
                        if let Some(mut buy_order) = queue.pop_front() {
                            let trade_qty = order.quantity.min(buy_order.quantity);
                            order.quantity -= trade_qty;
                            buy_order.quantity -= trade_qty;
                            trades.push(Trade { 
                                from: order.username.clone(), 
                                to: buy_order.username.clone(),
                                trade_qty, 
                                trade_price: highest_buy_price,
                                stock_type : buy_order.stock_type.clone()
                            });

                            if buy_order.quantity > 0 {
                                queue.push_front(buy_order);
                            }
                        } else {
                            self.buy.remove(&highest_buy_price);
                        }
                    } else {
                        break; // no matching buy
                    }
                }

                if order.quantity > 0 {
                    self.sell.entry(order.price).or_insert_with(VecDeque::new).push_back(order);
                }
            }
        }
        Ok(trades)
    }
    
    pub fn execute_market_order(&mut self , username : String , ordertype : Ordertype , mut quantity : u64 , user : &mut User , market_id : String , stock_type : StockType ) -> Result<Vec<Trade>, String> {
        let mut trades = vec![];
        match ordertype {
            Ordertype::Buy => {
                while quantity > 0 {
                    if let Some((&lowest_sell_price, queue)) = self.sell.iter_mut().next() {
                        if let Some(mut sell_order) = queue.pop_front() {
                            let trade_qty = quantity.min(sell_order.quantity);
                            if user.balance < trade_qty * lowest_sell_price{
                                break;
                            }
                            quantity -= trade_qty;
                            sell_order.quantity -= trade_qty;
                            user.balance -= trade_qty * lowest_sell_price;
                            trades.push(
                                Trade{
                                    from : sell_order.username.clone(),
                                    to : username.clone(),
                                    trade_qty,
                                    trade_price : lowest_sell_price,
                                    stock_type : sell_order.stock_type.clone()
                                }
                            );

                            if sell_order.quantity > 0 {
                                queue.push_front(sell_order);
                            }
                        } else {
                            self.sell.remove(&lowest_sell_price);
                        }
                    } else {
                         break; // no sells left
                    }
                }
            }

            Ordertype::Sell => {
                let holdings = user.holdings.entry(market_id.clone()).or_default();
                let available_stock = match stock_type {
                        StockType::StockA => &mut holdings.stock_a,
                        StockType::StockB => &mut holdings.stock_b,
                };
                // return error if user has less stock than he is selling
                 if *available_stock < quantity {
                    return Err(format!("Insufficient stock. Required: {}, Available: {}", quantity, available_stock));
                 }

                while quantity > 0 {
                    if let Some((&highest_buy_price, queue)) = self.buy.iter_mut().next_back() {
                        if let Some(mut buy_order) = queue.pop_front() {
                            let trade_qty = quantity.min(buy_order.quantity);
                           
                            quantity -= trade_qty;
                            buy_order.quantity -= trade_qty;
                            *available_stock -= trade_qty;  // lock the users stock 

                            trades.push(Trade { 
                                from: username.clone(), 
                                to: buy_order.username.clone(), 
                                trade_qty, 
                                trade_price: highest_buy_price,
                                stock_type : buy_order.stock_type.clone()
                            });

                            if buy_order.quantity > 0 {
                                queue.push_front(buy_order);
                            }
                        } else {
                            self.buy.remove(&highest_buy_price);
                        }
                    } else {
                        break; // no buys left
                    }
                }
            }
        }
       Ok(trades)
    }
}