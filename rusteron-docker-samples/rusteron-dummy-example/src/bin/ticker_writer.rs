use log::{info, warn};
use rusteron_archive::*;
use rusteron_dummy_example::model::Subscribe;
use rusteron_dummy_example::{
    archive_connect, download_ws, init_logger, JsonMesssageHandler, TICKER_CHANNEL,
    TICKER_STREAM_ID,
};
use std::fmt::Debug;
use std::time::Duration;
use tokio::time::Instant;

#[tokio::main]
async fn main() -> websocket_lite::Result<()> {
    init_logger();

    let pairs = vec![
        "btcusdt",
        "ethusdt",
        "bnbusdt",
        "ltcusdt",
        "solusdt",
        "dotusdt",
        "maticusdt",
        "avaxusdt",
        "nearusdt",
        "adausdt",
        "xrpusdt",
    ];

    let mut id = 0;
    let url = "wss://stream.binance.com/ws";

    let mut params = vec![];
    for pair in &pairs {
        params.push(format!("{pair}@ticker"));
    }

    let subscription = Subscribe {
        method: "SUBSCRIBE".to_string(),
        params,
        id,
    };

    let (archive, aeron) = archive_connect()?;

    let handle = tokio::spawn(download_ws(
        url,
        subscription.clone(),
        AeronRecorder::new(archive, aeron)?,
    ));

    handle.await??;
    Ok(())
}

struct AeronRecorder {
    publication: AeronExclusivePublication,
    published_count: usize,
}

impl AeronRecorder {
    pub fn new(archive: AeronArchive, aeron: Aeron) -> websocket_lite::Result<Self> {
        let channel = TICKER_CHANNEL;
        let stream_id = TICKER_STREAM_ID;
        let subscription_id =
            archive.start_recording(channel, stream_id, SOURCE_LOCATION_REMOTE, true)?;
        info!("started recording ticker stream [subscriptionId={subscription_id}");

        let publication = aeron
            .async_add_exclusive_publication(channel, stream_id)?
            .poll_blocking(Duration::from_secs(60))?;

        info!(
            "created exclusive ticker publication [sessionId={}]",
            publication.get_constants()?.session_id
        );

        Ok(Self {
            publication,
            published_count: 0,
        })
    }
}

impl JsonMesssageHandler for AeronRecorder {
    fn on_msg(&mut self, msg: &str) {
        let mut result = self.publication.offer(
            msg.as_bytes(),
            Handlers::no_reserved_value_supplier_handler(),
        );
        if result <= 0 {
            // this is poor way to handle back pressure, just for simple example
            let duration = Duration::from_millis(100);
            let start = Instant::now();

            while start.elapsed() < duration && result <= 0 {
                result = self.publication.offer(
                    msg.as_bytes(),
                    Handlers::no_reserved_value_supplier_handler(),
                );
            }

            if result <= 0 {
                warn!(
                    "failed to publish [error={:?}, payload={}]",
                    AeronCError::from_code(result as i32),
                    msg
                )
            }
        }

        if result > 0 {
            self.published_count += 1;

            if self.published_count % 1000 == 0 {
                info!("published {} ticker messages so far", self.published_count);
            }
        }
    }
}
