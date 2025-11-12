use crate::{ User, order::*};
use nanoid::nanoid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WinningOutcome {
    OutcomeA,
    OutcomeB,
    Neither, // Draw or invalid outcome - both tokens get 50% payout
}

#[derive(Debug)]
pub struct Market {
    pub market_id : String,
    pub created_by : String,
    pub market_name : String,
    pub stock_a: OrderBook,
    pub stock_b: OrderBook,
    pub trades : Vec<Trade>,
    pub winning_outcome : Option<WinningOutcome>,
    pub is_settled : bool
}

impl Market {
    pub fn initialise_market (market_name : String , username : String) -> Self{ 
        let market = Market{
            market_id : nanoid!(),
            created_by : username,
            market_name ,
            stock_a : OrderBook::new(),
            stock_b : OrderBook::new(),
            trades : vec![],
            winning_outcome : None,
            is_settled : false
        };
        market
    }
    pub fn add_limit_order(&mut self , mut order : Order , user : &mut User) -> Result<Vec<Trade> , String> {
        match order.stock_type {
            StockType::StockA => { 
                let mut v  =  self.stock_a.add_limit_order(order, user);
                match v {
                    Ok(mut v) => {
                       let v2 = v.clone();
                       self.trades.append(&mut v);
                       Ok(v2)
                    }
                    Err(err) => {
                        Err(err)
                    }
                }
            }
            StockType::StockB => {
               let mut v  =  self.stock_b.add_limit_order(order, user );
                match v {
                    Ok(mut v) => {
                       let v2 = v.clone();
                       self.trades.append(&mut v);
                       Ok(v2)
                    }
                    Err(err) => {
                        Err(err)
                    }
                }
            }
        }
        
    }
    pub fn execute_market_order(&mut self , username : String , ordertype : Ordertype , quantity : u64, stock_type : StockType , user : &mut User , market_id : String  )-> Result<Vec<Trade> , String> {
        let mut v = match stock_type {
            StockType::StockA => {
                self.stock_a.execute_market_order(username, ordertype, quantity , user , market_id , stock_type)
            }
            StockType::StockB => {
                self.stock_b.execute_market_order(username, ordertype, quantity , user  , market_id , stock_type)
            }
        };
        match v {
                Ok(mut v) => {
                    let v2 = v.clone();
                    self.trades.append(&mut v);
                    Ok(v2)
                }
                Err(err) => {
                    Err(err)
                }
            }
    }
}