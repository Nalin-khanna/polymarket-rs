use exchange_rs::{
    Orderbooks, Ordertype, StockType, UserDetails, models::request::Request, utils::hash_password, worker::processor::spawn_background_worker 
};
use tokio::sync::{mpsc::Sender, oneshot};

async fn signup_user(
    tx: &Sender<Request>,
    user: &str,
    pass: &str,
) -> Result<String, String> {
    let (resp_tx, resp_rx) = oneshot::channel();
    // Mimic the route: hash the password before sending
    let hashed_pass = hash_password(pass);

    let req = Request::Signup {
        username: user.to_string(),
        password: hashed_pass,
        resp: resp_tx,
    };
    tx.send(req).await.expect("Test worker send failed");
    resp_rx.await.expect("Test worker response failed")
}

async fn signin_user(
    tx: &Sender<Request>,
    user: &str,
    pass: &str,
) -> Result<String, String> {
    let (resp_tx, resp_rx) = oneshot::channel();
    
    let req = Request::Signin {
        username: user.to_string(),
        password: pass.to_string(), // Send plaintext, as per your route
        resp: resp_tx,
    };

    tx.send(req).await.expect("Test worker send failed");
    resp_rx.await.expect("Test worker response failed")
}

async fn new_market(
    tx: &Sender<Request>,
    user: &str,
    market_name: &str,
) -> Result<String , String> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let req = Request::CreateMarket { username: user.to_string(), market_name: market_name.to_string(), resp: resp_tx };
    tx.send(req).await.expect("Test worker send failed");
    resp_rx.await.expect("Test worker response failed")
}

async fn split_stocks (
    tx: &Sender<Request>,
    user: &str,
    market_id: &str,
    amount : u64
) -> Result<String, String> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let req = Request::SplitStocks { username: user.to_string(), market_id: market_id.to_string(), amount, resp: resp_tx };
    tx.send(req).await.expect("Test worker send failed");
    resp_rx.await.expect("Test worker response failed")
}

async fn get_user_details (
    tx : &Sender<Request>,
    username :  &str,
)-> Result<UserDetails, String>{
    let (resp_tx, resp_rx) = oneshot::channel();
    let req = Request::UserDetails { username: username.to_string(), resp: resp_tx };
    tx.send(req).await.expect("Test worker send failed");
    resp_rx.await.expect("Test worker response failed")
}

async fn limit_order (
    tx : &Sender<Request>,
    username : &str, 
    stock_type: StockType, 
    price: u64, 
    quantity:u64,
    market_id : &str,
    ordertype: Ordertype, 
)-> Result<String, String> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let req = Request::CreateLimitOrder { 
        username: username.to_string(), 
        stock_type, 
        price, 
        quantity, 
        ordertype, 
        market_id: market_id.to_string(), 
        resp: resp_tx };
    tx.send(req).await.expect("Test worker send failed");
    resp_rx.await.expect("Test worker response failed")
}

async fn get_orderbook (
    tx : &Sender<Request>,
    market_id : &str,
) -> Result<Orderbooks, String>{
    let (resp_tx, resp_rx) = oneshot::channel();
    let req = Request::GetOrderbook { 
        market_id: market_id.to_string() , 
        resp: resp_tx 
    };
    tx.send(req).await.expect("Test worker send failed");
    resp_rx.await.expect("Test worker response failed")
}

#[tokio::test]
async fn test_auth_flow() {
    //  Start the worker
    let tx = spawn_background_worker();

    //  Test successful signup
    let res_ok = signup_user(&tx, "user1", "pass123").await;
    assert_eq!(res_ok, Ok("user1".to_string()));

    // second user signup 
    let res_ok = signup_user(&tx, "user2", "pass345").await;
    assert_eq!(res_ok, Ok("user2".to_string()));

    // second user signin 
    let res_correct_pass = signin_user(&tx, "user2", "pass345").await;
    assert_eq!(res_correct_pass, Ok("user2".to_string()));

    //  Test duplicate signup
    let res_dup = signup_user(&tx, "user1", "pass456").await;
    assert!(res_dup.is_err());
    assert!(res_dup
        .unwrap_err()
        .contains("Username already exists"));

    //  Test signin with wrong password
    let res_wrong_pass = signin_user(&tx, "user1", "wrongpass").await;
    assert!(res_wrong_pass.is_err());
    assert_eq!(res_wrong_pass.unwrap_err(), "Invalid password");

    //  Test signin with correct password
    let res_correct_pass = signin_user(&tx, "user1", "pass123").await;
    assert_eq!(res_correct_pass, Ok("user1".to_string()));

    // check balance of user after signup 
    let res_bal = get_user_details(&tx, "user1").await;
    assert!(res_bal.is_ok(), "Getting user details failed: {:?}", res_bal.err());
    let test_amount = 5000 ;
    let details = res_bal.unwrap();
    assert_eq!(details.balance, test_amount);

    //  Test signin with non-existent user
    let res_no_user = signin_user(&tx, "user_does_not_exist", "pass123").await;
    assert!(res_no_user.is_err());
    assert_eq!(res_no_user.unwrap_err(), "User not found");

    //test create market 
    let res_market = new_market(&tx, "user1", "market_name").await;
    assert!(res_market.is_ok());
    let market_id = res_market.unwrap();
    assert!(!market_id.is_empty());

    // test create market with non-existing user 
    let res_no_market = new_market(&tx, "user3", "market_name").await;
    assert!(res_no_market.is_err());

    // test split stocks 
    let res_split = split_stocks(&tx, "user2", &market_id, 100).await;
    assert_eq!(res_split, Ok("Minted 100 of Stock A and B".to_string()));

    // check user user holdings after split 
    let res_bal = get_user_details(&tx, "user2").await;
    assert!(res_bal.is_ok(), "Getting user details failed: {:?}", res_bal.err());
    let details2 = res_bal.unwrap();
    assert!(details2.holdings.get(&market_id).is_some());
    let holdings = details2.holdings.get(&market_id).unwrap();
    assert!(holdings.stock_a == 100 && holdings.stock_b == 100);
    assert!(details2.balance == test_amount -100);

    // split stocks for balance more than user's balance 
    let res_split = split_stocks(&tx, "user2", &market_id, test_amount).await;
    assert_eq!(res_split, Err("Insufficient funds to mint".to_string()));

    // Get Orderbook (Empty) 
    let orderbook = get_orderbook(&tx, &market_id).await.unwrap();
    let stock_a_orderbook = orderbook.stock_a;
    let stock_b_orderbook = orderbook.stock_b;
    assert!(stock_a_orderbook.buy.is_empty());
    assert!(stock_b_orderbook.buy.is_empty());
    assert!(stock_a_orderbook.sell.is_empty());
    assert!(stock_b_orderbook.sell.is_empty());

    // test create limit sell order 
    let res_limit_sell = limit_order(&tx, "user2", StockType::StockA, 50, 10, &market_id, Ordertype::Sell).await;
    assert_eq!(res_limit_sell , Ok("Order placed, waiting to be matched.".to_string()));

    // create limit buy 
    let res_limit_buy = limit_order(&tx, "user1", StockType::StockA, 40, 5, &market_id, Ordertype::Buy).await;
    assert_eq!(res_limit_buy , Ok("Order placed, waiting to be matched.".to_string()));

     //Verify Orderbook & Balances (Locked)
    let ob_after = get_orderbook(&tx, &market_id).await.unwrap();
    let stock_a_ob = ob_after.stock_a;
    assert_eq!(stock_a_ob.buy.len(), 1); // user1's buy order
    assert_eq!(stock_a_ob.sell.len(), 1); // user2's sell order

    let u1_locked = get_user_details(&tx, "user1").await.unwrap();
    // user1's balance is locked
    assert_eq!(u1_locked.balance, 5000 - (40 * 5)); 
    assert!(u1_locked.holdings.get(&market_id).is_none()); // stocks unchanged

    let u2_locked = get_user_details(&tx, "user2").await.unwrap();
    // user2's stocks are locked
    assert_eq!(u2_locked.balance, 4900); // balance unchanged
    assert_eq!(u2_locked.holdings.get(&market_id).unwrap().stock_a, 90);

    // limit buy order by user1 which will match the sell order 
    let trades = limit_order(&tx, "user1", StockType::StockA, 60, 5, &market_id, Ordertype::Buy).await.unwrap();
    assert!(trades.starts_with("[Trade"), "Expected a trade string, got: {}", trades);
    assert!(trades.contains("from: \"user2\""));
    assert!(trades.contains("to: \"user1\""));
    assert!(trades.contains("trade_qty: 5"));
    assert!(trades.contains("trade_price: 50"));

    // orderbook still contains sell of 5 A stocks from user2
    let ob_after = get_orderbook(&tx, &market_id).await.unwrap();
    let stock_a_ob = ob_after.stock_a; 
    assert_eq!(stock_a_ob.sell.len(), 1);

    // checking balances and stocks after trade execution
    let u1 = get_user_details(&tx, "user1").await.unwrap();
    assert_eq!(u1.balance, 5000 - (40 * 5) - (50*5)); // 1 open buy order and 1 executed
    assert_eq!(u1.holdings.get(&market_id).unwrap().stock_a , 5); // stock holdings increased

    let u2 = get_user_details(&tx, "user2").await.unwrap();
    assert_eq!(u2.balance, 4900 + (50*5)); // seller's balance increased after trade

    // limit buy order by user1 which will eat the sell orderbook
    let trades = limit_order(&tx, "user1", StockType::StockA, 60, 10, &market_id, Ordertype::Buy).await.unwrap();
    assert!(trades.starts_with("[Trade"), "Expected a trade string, got: {}", trades);
    assert!(trades.contains("from: \"user2\""));
    assert!(trades.contains("to: \"user1\""));
    assert!(trades.contains("trade_qty: 5"));
    assert!(trades.contains("trade_price: 50"));

    // orderbook contains no sell orders
    let ob_after = get_orderbook(&tx, &market_id).await.unwrap();
    let stock_a_ob = ob_after.stock_a; 
    assert_eq!(stock_a_ob.sell.len(), 0);
    assert_eq!(stock_a_ob.buy.len(), 2);  //another buy order placed for the 5 stocks
   
}