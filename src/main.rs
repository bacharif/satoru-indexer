use starknet::{
    core::types::{BlockId, BlockTag, FieldElement, EventFilter},
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider
    },
};
use starknet::macros::selector;
use dotenv::dotenv;
use url::Url;
use tokio_postgres::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let (client, connection) =
        tokio_postgres::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {:?}", e);
        }
    });

    let provider = JsonRpcClient::new(HttpTransport::new(
        Url::parse("https://ancient-skilled-asphalt.strk-sepolia.quiknode.pro/ab96caa503ba84b2f1631ccf7db3f15380314ed7").unwrap(),
    ));
    
    let order_created_key = FieldElement::from_hex_be("03427759bfd3b941f14e687e129519da3c9b0046c5b9aaa290bb1dede63753b3").unwrap();

    let event_filter = EventFilter {
        from_block: Some(BlockId::Number(64539)),
        to_block: Some(BlockId::Tag(BlockTag::Latest)),
        address: FieldElement::from_hex_be("0x2cf721c0387704095d6b2205b46e17d7768fa55c2f1a1087425b877b72937db").ok(),
        keys: Some(vec![vec![order_created_key]])
    };

    let events_result = provider.get_events(event_filter, None, 100).await;

    match events_result {
        Ok(events_page) => {
            for event in events_page.events {
                println!("Event found: {:?}", event);
                let block_number = event.block_number as i32;
                let transaction_hash_bytes = event.transaction_hash.to_bytes_be();  
                let transaction_hash = hex::encode(transaction_hash_bytes);
                let data = event.data.iter()
                    .map(|fe| hex::encode(fe.to_bytes_be()))
                    .collect::<Vec<_>>()
                    .join(",");

                client.execute(
                    "INSERT INTO orders (block_number, transaction_hash, data) VALUES ($1, $2, $3)",
                    &[&block_number, &transaction_hash, &data],
                ).await?;
            }
        },
        Err(e) => {
            println!("Failed to fetch events: {:?}", e);
        }
    }
    Ok(())
}
