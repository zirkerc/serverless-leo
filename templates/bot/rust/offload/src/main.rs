use std::time::Duration;
use serde::{Deserialize, Serialize};
use futures::FutureExt;
use rstreams::{events::Event, BotInvocationEvent, LeoCheckpointOptions, LeoReadOptions, LeoSdk, AllProviders, Error};
use rstreams::aws::AWSProvider;
use rusoto_signature::Region;
use lambda::{run, handler_fn};

pub struct ExampleSdkConfig;

impl ExampleSdkConfig {
    #[allow(dead_code)]
    pub fn dsco_test_bus() -> AllProviders {
        AllProviders::AWS(AWSProvider::new(
            Region::UsEast1,
            "TestBus-LeoStream-R2VV0EJ6FRI9",
            "TestBus-LeoCron-OJ8ZNCEBL8GM",
            "TestBus-LeoEvent-FNSO733D68CR",
            "testbus-leos3-1erchsf3l53le",
            "TestBus-LeoKinesisStream-1XY97YYPDLVQS",
            "TestBus-LeoFirehoseStream-1M8BJL0I5HQ34",
        ))
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct MyReadEvent {
    suborder_id: usize,
    order_created: String,
    number_of_line_items: usize,
    po_number: String,
    order_status: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler_fn(offload)).await?;
    Ok(())
}


async fn offload(event: BotInvocationEvent<()>, context: lambda::Context) -> Result<(), Error> {

    let sdk = LeoSdk::new(ExampleSdkConfig::dsco_test_bus());

    let bot_id = &event.bot_id;
    let source_queue = "SOURCE_TOKEN";

    sdk.cron(&event.__cron, &context, || async {
        sdk.offload(
            bot_id,
            source_queue,
            LeoReadOptions::default(),
            LeoCheckpointOptions::Enabled.with_initial_values(&event),
            |event: Event<MyReadEvent>| async move {
                let results = processing_function(event.eid.clone()).await;

                match results {
                    Ok(..) => Some(Ok(())),
                    Err(err) => {
                        Some(Err(anyhow!(format!("Failed to process event, will not checkpoint {}", err))))
                    }
                }
            },
        )
            .await
            .map_err(|e| e.into())
    })
        .then(|r| async move {
            println!("Handler done!");
            r
        })
        .await?;

    Ok(())
}

async fn processing_function(eid: String) -> Result<()> {
    println!("Handling event eid: {}", eid);
    Ok(())
}