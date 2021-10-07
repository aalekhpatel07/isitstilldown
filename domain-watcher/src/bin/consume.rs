use domain_watcher::kafka;
use std::sync::Arc;
use tokio;
use tokio::{sync::mpsc, task, time, time::Duration};
use log::{info, debug};




/// Handle the message subscription command.
///
/// This will subscribe to a kafka-topic on which metrics are being published.
/// Then the incoming message is deserialized back to BatchMessage and
/// published to an internal channel.
/// Then this data is read and published to postgres.
///
async fn handle_message_receiving(config: Arc<kafka::Config>) {
	let (tx, mut rx) = mpsc::channel(100);
	task::spawn(async move {
		info!("Waiting to receive metrics-data on incoming queue.");
		while let Some(data) = rx.recv().await {
			debug!("Received data on the incoming channel;");
            println!("{:#?}", data);
//			if let Ok(bmsg) = BatchMessage::decode(raw_data) {
//				if let Err(e) = dbclient.insert(&bmsg).await {
//					error!("Failed to write data to the db: {:?}", e);
//					let _ = dbclient.insert(&bmsg).await;
//				}
//			} else {
//				error!("Failed to decode the incoming message from kafka");
//			};
		}
	});

	debug!("Starting to cosume the data");
	let conf = config.clone();
	let kconsumer = kafka::create_consumer(conf);
	kconsumer.consume(tx).await;
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let app_config = Arc::new(kafka::Config::new());

    handle_message_receiving(app_config).await;
    Ok(())
}
