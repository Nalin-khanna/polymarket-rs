
use chrono::{DateTime, Utc};
use std::collections::{ BTreeMap , VecDeque};
#[derive(Debug)]
pub struct Order {
    price : u64, 
    quantity : u64,
    option: Option,
    username : String,
    timestamp: DateTime<Utc>,
    ordertype: Ordertype,
    
}
#[derive(Debug)]
pub enum Option {
    OptionA ,
    OptionB 
}
#[derive(Debug , Clone)]
pub enum Ordertype{
    Buy,
    Sell
}
#[derive(Debug)]
pub struct OrderBook {
    buy: BTreeMap<u64, VecDeque<Order>>,
    sell: BTreeMap<u64, VecDeque<Order>>,
}
impl OrderBook {
    pub fn add_new_order(
        &mut self,
        price : u64 ,
        quantity: u64,
        option : Option,
        ordertype : Ordertype,
        username : String,

    ){
        let order = Order{price , quantity, option , username , timestamp : Utc::now() , ordertype : ordertype.clone()};
        match ordertype {
            Ordertype::Buy => {
                self.buy.entry(price).or_insert_with(VecDeque::new).push_back(order);
            }
            Ordertype::Sell => {
                self.sell.entry(price).or_insert_with(VecDeque::new).push_back(order);
            }
        }
    }
}