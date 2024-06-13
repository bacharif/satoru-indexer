use starknet::{
    core::types::{BlockId, BlockTag, FieldElement, EventFilter},
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider
    },
};
use dotenv::dotenv;
use url::Url;
use tokio_postgres::{NoTls, Error};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let (client, connection) =
        tokio_postgres::connect(&env::var("DATABASE_URL").unwrap(), NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {:?}", e);
        }
    });

    let provider = JsonRpcClient::new(HttpTransport::new(
        Url::parse("https://ancient-skilled-asphalt.strk-sepolia.quiknode.pro/ab96caa503ba84b2f1631ccf7db3f15380314ed7").unwrap(),
    ));
    
    let order_created_key = FieldElement::from_hex_be("03427759bfd3b941f14e687e129519da3c9b0046c5b9aaa290bb1dede63753b3").unwrap();
    let deposit_created_key = FieldElement::from_hex_be("00ee02d31cafad9001fbdc4dd5cf4957e152a372530316a7d856401e4c5d74bd").unwrap();
    let withdrawal_created_key = FieldElement::from_hex_be("02021e2242f6c652ae824bc1428ee0fe7e8771a27295b9450792445dc456e37d").unwrap();

    let event_filter = EventFilter {
        from_block: Some(BlockId::Number(64539)),
        to_block: Some(BlockId::Tag(BlockTag::Latest)),
        address: FieldElement::from_hex_be("0x2cf721c0387704095d6b2205b46e17d7768fa55c2f1a1087425b877b72937db").ok(),
        keys: Some(vec![vec![order_created_key, deposit_created_key, withdrawal_created_key]])
    };

    let events_result = provider.get_events(event_filter, None, 100).await;

    match events_result {
        Ok(events_page) => {
            for event in events_page.events {
                println!("Event found: {:?}", event);
                let block_number = event.block_number as i64;  // Use i64 to match BIGINT in PostgreSQL
                let transaction_hash_bytes = event.transaction_hash.to_bytes_be();  
                let transaction_hash = hex::encode(transaction_hash_bytes);
                let key = event.keys.first().map(|k| hex::encode(k.to_bytes_be()));
                let data = event.data.iter()
                    .map(|fe| hex::encode(fe.to_bytes_be()))
                    .collect::<Vec<_>>()
                    .join(",");

                if event.keys.contains(&order_created_key) {
                    // Insert into orders
                    client.execute(
                        "INSERT INTO orders (
                            block_number, transaction_hash, key, order_type, decrease_position_swap_type, account,
                            receiver, callback_contract, ui_fee_receiver, market, initial_collateral_token, swap_path,
                            size_delta_usd, initial_collateral_delta_amount, trigger_price, acceptable_price,
                            execution_fee, callback_gas_limit, min_output_amount, updated_at_block, is_long, is_frozen
                        ) VALUES (
                            $1, $2, $3, NULL, NULL, NULL,
                            NULL, NULL, NULL, NULL, NULL, NULL,
                            NULL, NULL, NULL, NULL,
                            NULL, NULL, NULL, NULL, NULL, NULL
                        )",
                        &[&block_number, &transaction_hash, &key],
                    ).await?;
                } else if event.keys.contains(&deposit_created_key) {
                    // Insert into deposits
                    client.execute(
                        "INSERT INTO deposits (
                            block_number, transaction_hash, key, account, receiver, callback_contract,
                            market, initial_long_token, initial_short_token, long_token_swap_path, short_token_swap_path,
                            initial_long_token_amount, initial_short_token_amount, min_market_tokens, updated_at_block,
                            execution_fee, callback_gas_limit
                        ) VALUES (
                            $1, $2, $3, NULL, NULL, NULL,
                            NULL, NULL, NULL, NULL, NULL,
                            NULL, NULL, NULL, NULL,
                            NULL, NULL
                        )",
                        &[&block_number, &transaction_hash, &key],
                    ).await?;
                } else if event.keys.contains(&withdrawal_created_key) {
                    // Insert into withdrawals
                    client.execute(
                        "INSERT INTO withdrawals (
                            block_number, transaction_hash, key, account, receiver, callback_contract,
                            market, long_token_swap_path, short_token_swap_path, market_token_amount,
                            min_long_token_amount, min_short_token_amount, updated_at_block, execution_fee,
                            callback_gas_limit
                        ) VALUES (
                            $1, $2, $3, NULL, NULL, NULL,
                            NULL, NULL, NULL, NULL,
                            NULL, NULL, NULL, NULL,
                            NULL
                        )",
                        &[&block_number, &transaction_hash, &key],
                    ).await?;
                } else {
                    println!("Unknown event type: {:?}", event);
                }
            }
        },
        Err(e) => {
            println!("Failed to fetch events: {:?}", e);
        }
    }
    Ok(())
}
