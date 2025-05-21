mod blockchain;
mod block;
mod transaction;
mod wallet;
mod errors;

use blockchain::Blockchain;
use transaction::{Transaction, TransactionType};

fn main() {
    // PoW, difficulty level = 2, mining reward = 100
    let mut my_chain = Blockchain::new(2, 100.0, blockchain::ConsensusAlgorithm::ProofOfWork);
    
    my_chain.create_wallet(String::from("alice"));
    my_chain.create_wallet(String::from("bob"));
    my_chain.create_wallet(String::from("miner"));
    
    println!("--Initial balances:");

    my_chain.add_funds_to_wallet("alice", 1000.0).unwrap();
    my_chain.add_funds_to_wallet("bob", 500.0).unwrap();
    
    println!("Alice: {}", my_chain.get_balance("alice"));
    println!("Bob: {}", my_chain.get_balance("bob"));
    println!("Miner: {}", my_chain.get_balance("miner"));
    
    println!("\n--Adding test transaction...");
    match my_chain.add_transaction(Transaction::new(
        String::from("alice"),
        String::from("bob"),
        50.0,
        TransactionType::Transfer
    )) {
        Ok(_) => println!("Transaction added to pendings"),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\n--Mining block...");
    match my_chain.mine_pending_transactions(String::from("miner")) {
        Ok(_) => println!("Block added to chain"),
        Err(e) => println!("Mining error: {}", e),
    }
    
    println!("\n--Balances after transaction:");
    println!("Alice: {}", my_chain.get_balance("alice"));
    println!("Bob: {}", my_chain.get_balance("bob"));
    println!("Miner: {}", my_chain.get_balance("miner"));
    
    println!("\n--Another transaction...");
    match my_chain.add_transaction(Transaction::new(
        String::from("bob"),
        String::from("alice"),
        20.0,
        TransactionType::Transfer
    )) {
        Ok(_) => println!("Transaction added to pendings"),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\n--Mining block...");
    match my_chain.mine_pending_transactions(String::from("miner")) {
        Ok(_) => println!("Block added to chain"),
        Err(e) => println!("Mining error: {}", e),
    }
    
    
    println!("\n--Balances after transaction:");
    println!("Alice: {}", my_chain.get_balance("alice"));
    println!("Bob: {}", my_chain.get_balance("bob"));
    println!("Miner: {}", my_chain.get_balance("miner"));
    
    println!("\n--Creating smart contract...");
    match my_chain.create_smart_contract(
        String::from("alice"),
        String::from("function transfer() { return 'transfer executed'; }"),
        10.0
    ) {
        Ok(address) => {
            println!("Smart contract created. Its address: {}", address);
            
            println!("\nMining block with smart contract...");
            match my_chain.mine_pending_transactions(String::from("miner")) {
                Ok(_) => {
                    println!("Block added");
                    
                    println!("\nRunning smart contract...");
                    match my_chain.execute_smart_contract(&address, "transfer", vec![]) {
                        Ok(result) => println!("Result is: {}", result),
                        Err(e) => println!("Error: {}", e),
                    }
                },
                Err(e) => println!("Error when mining: {}", e),
            }
        },
        Err(e) => println!("Cannot create smart contract: {}", e),
    }
    
    println!("\n--Bob saves some data in blockchain as a transaction...");
    match my_chain.store_data(
        String::from("bob"),
        "Some important data".as_bytes().to_vec()
    ) {
        Ok(data_id) => {
            println!("Data stored with ID: {}", data_id);
            
            println!("\nMining a block with data...");
            match my_chain.mine_pending_transactions(String::from("miner")) {
                Ok(_) => println!("Success"),
                Err(e) => println!("Error: {}", e),
            }
        },
        Err(e) => println!("Cannot save data: {}", e),
    }
    
    println!("\n--Let now create another blockchain. It will use Proof of Stake...");
    let mut pos_chain = Blockchain::new(1, 50.0, blockchain::ConsensusAlgorithm::ProofOfStake);
    
    pos_chain.create_wallet(String::from("validator1"));
    pos_chain.create_wallet(String::from("validator2"));
    pos_chain.create_wallet(String::from("justuser"));
    
    pos_chain.add_funds_to_wallet("validator1", 1000.0).unwrap();
    pos_chain.add_funds_to_wallet("validator2", 2000.0).unwrap();
    pos_chain.add_funds_to_wallet("justuser", 500.0).unwrap();
    
    println!("--Registering validators...");
    match pos_chain.add_validator(String::from("validator1"), 800.0) {
        Ok(_) => println!("validator1 registered with stake 800.0"),
        Err(e) => println!("Error: {}", e),
    }
    
    match pos_chain.add_validator(String::from("validator2"), 1500.0) {
        Ok(_) => println!("validator1 registered with stake 1500.0"),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\n--Adding transaction in PoS...");
    match pos_chain.add_transaction(Transaction::new(
        String::from("justuser"),
        String::from("validator1"),
        25.0,
        TransactionType::Transfer
    )) {
        Ok(_) => println!("Transaction added"),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\n--Validation block in PoS...");
    match pos_chain.mine_pending_transactions(String::from("validator2")) {
        Ok(_) => println!("Block validated and added into chain"),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\n--Balances in PoS blockchain:");
    println!("validator1: {}", pos_chain.get_balance("validator1"));
    println!("validator2: {}", pos_chain.get_balance("validator2"));
    println!("justuser: {}", pos_chain.get_balance("justuser"));
    
    if let Some(wallet) = pos_chain.get_wallet_info("validator2") {
        println!("\nvalidator2 wallet:");
        println!("Address: {}", wallet.address);
        println!("Balance: {}", wallet.balance);
        println!("Staking balance: {}", wallet.staking_balance);
        println!("Transaction count: {}", wallet.transaction_history.len());
    }
    
    println!("\nChecking chain validity:");
    println!("PoW chain: {}", my_chain.is_chain_valid());
    println!("PoS chain: {}", pos_chain.is_chain_valid());
    
    my_chain.adjust_difficulty();
    
    println!("\nAll blocks in PoW chain:");
    for block in &my_chain.chain {
        println!("{:?}", block);
    }
    
    println!("\nAll blocks в PoS chain:");
    for block in &pos_chain.chain {
        println!("{:?}", block);
    }
    
    println!("\nTests done!");
}