use hdrhistogram::Histogram;
use rusteron_client::*;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;

const PING_STREAM_ID: i32 = 1002;
const PONG_STREAM_ID: i32 = 1003;
const PING_CHANNEL: &str = "aeron:udp?endpoint=localhost:20123";
const PONG_CHANNEL: &str = "aeron:udp?endpoint=localhost:20124";
const NUMBER_OF_MESSAGES: usize = 10_000_000;
const WARMUP_NUMBER_OF_MESSAGES: usize = 100_000;
const MESSAGE_LENGTH: usize = 32;
const FRAGMENT_COUNT_LIMIT: usize = 10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let running_pong = Arc::clone(&running);

    // Set up the pong thread
    let pong_thread = thread::Builder::new()
        .name("pong".to_string())
        .spawn(move || run_pong(running_pong).unwrap())?;

    let hist = run_ping(running, pong_thread)?;
    println!("message length {} bytes\n", MESSAGE_LENGTH);
    println!("Histogram of RTT latencies in micros:");
    println!("# of samples: {}", hist.len());
    println!("min: {}", hist.min() as f64 / 1000.0);
    println!(
        "50th percentile: {}",
        hist.value_at_quantile(0.50) as f64 / 1000.0
    );
    println!(
        "99th percentile: {}",
        hist.value_at_quantile(0.99) as f64 / 1000.0
    );
    println!(
        "99.9th percentile: {}",
        hist.value_at_quantile(0.999) as f64 / 1000.0
    );
    println!("max: {}", hist.max() as f64 / 1000.0);
    println!("avg: {}", hist.mean() / 1000.0);

    Ok(())
}

fn run_pong(running_pong: Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error>> {
    let context = AeronContext::new()?;
    let dir = std::env::var("AERON_DIR").expect("AERON_DIR must be set");
    context.set_dir(&dir)?;
    let aeron = Aeron::new(&context)?;
    aeron.start()?;
    let ping_publication = aeron
        .async_add_publication(PING_CHANNEL, PING_STREAM_ID)?
        .poll_blocking(Duration::from_secs(5))?;
    let pong_subscription = aeron
        .async_add_subscription(
            PONG_CHANNEL,
            PONG_STREAM_ID,
            Handlers::no_available_image_handler(),
            Handlers::no_unavailable_image_handler(),
        )?
        .poll_blocking(Duration::from_secs(4))?;

    println!("PONG: ping publisher {} {}", PING_CHANNEL, PING_STREAM_ID);
    println!("PONG: pong subscriber {} {}", PONG_CHANNEL, PONG_STREAM_ID);

    println!("Starting pong thread");
    pub struct PongRoundTripHandler {
        publisher: AeronPublication,
        buffer_claim: AeronBufferClaim,
        header_values: AeronHeaderValues,
    }

    impl AeronFragmentHandlerCallback for PongRoundTripHandler {
        #[inline]
        fn handle_aeron_fragment_handler(&mut self, buffer: &[u8], header: AeronHeader) {
            header.values(&self.header_values).unwrap();
            let flags = self.header_values.frame.flags;

            while self.publisher.try_claim(buffer.len(), &self.buffer_claim) < 0 {}
            self.buffer_claim.frame_header_mut().flags = flags;
            self.buffer_claim.data_mut().copy_from_slice(buffer);
            self.buffer_claim.commit().unwrap();

            // while self.publisher.offer(buffer, Handlers::no_reserved_value_supplier_handler()) < 0 {}
        }
    }

    let handler = Handler::leak(PongRoundTripHandler {
        publisher: ping_publication.clone(),
        buffer_claim: Default::default(),
        header_values: Default::default(),
    });
    while running_pong.load(Ordering::Acquire) {
        let _ = pong_subscription.poll(Some(&handler), FRAGMENT_COUNT_LIMIT);
    }
    println!("Shutting down pong thread");
    Ok(())
}

fn run_ping(
    running: Arc<AtomicBool>,
    pong_thread: JoinHandle<()>,
) -> Result<Histogram<u64>, Box<dyn Error>> {
    let context = AeronContext::new()?;
    let dir = std::env::var("AERON_DIR").expect("AERON_DIR must be set");
    context.set_dir(&dir)?;
    let aeron = Aeron::new(&context)?;
    aeron.start()?;

    let pong_publication = aeron
        .async_add_publication(PONG_CHANNEL, PONG_STREAM_ID)?
        .poll_blocking(Duration::from_secs(5))?;
    let ping_subscription = aeron
        .async_add_subscription(
            PING_CHANNEL,
            PING_STREAM_ID,
            Handlers::no_available_image_handler(),
            Handlers::no_unavailable_image_handler(),
        )?
        .poll_blocking(Duration::from_secs(4))?;

    println!("PING: pong publisher {} {}", PONG_CHANNEL, PONG_STREAM_ID);
    println!("PING: ping subscriber {} {}", PING_CHANNEL, PING_STREAM_ID);

    // Set up the ping thread
    let mut buffer = vec![0u8; MESSAGE_LENGTH];

    let mut handler = Handler::leak(PingRoundTripHandler {
        histogram: Histogram::new(3)?,
    });
    sleep(Duration::from_secs(1));
    let image = ping_subscription.image_at_index(0);
    for _ in 0..WARMUP_NUMBER_OF_MESSAGES {
        record_rtt(&pong_publication, &image, &mut buffer, &mut handler);
    }
    println!("warmed up");
    handler.histogram.reset();
    for _ in 0..NUMBER_OF_MESSAGES {
        record_rtt(&pong_publication, &image, &mut buffer, &mut handler);
    }

    println!("finished sending all pings");
    // Wait for thread to finish
    running.store(false, Ordering::SeqCst);
    pong_thread.join().expect("Failed to join pong thread");

    // Report histogram results

    let hist = &handler.histogram;
    Ok(hist.clone())
}

pub struct PingRoundTripHandler {
    histogram: Histogram<u64>,
}

impl AeronFragmentHandlerCallback for PingRoundTripHandler {
    #[inline]
    fn handle_aeron_fragment_handler(&mut self, buffer: &[u8], _header: AeronHeader) {
        let time = unsafe {
            let int_ptr = buffer.as_ptr()
                // .add(offset)
                as *const i64;
            i64::from_le(*int_ptr)
        };
        // println!("ping received {} {:?}", time, buffer);
        let rtt = Aeron::nano_clock() - time;
        // println!("RTT: {}", rtt);
        debug_assert!(rtt >= 0);
        self.histogram.record(rtt as u64).unwrap();
    }
}

#[inline]
fn record_rtt(
    pong_publication: &AeronPublication,
    image: &AeronImage,
    buffer: &mut Vec<u8>,
    handler: &mut Handler<PingRoundTripHandler>,
) {
    let now = Aeron::nano_clock();
    unsafe {
        std::ptr::copy_nonoverlapping(now.to_le_bytes().as_ptr(), buffer.as_mut_ptr(), 8);
    }
    let mut offered_position = -1;
    // println!("time is {} {:?}", i, buffer);
    while offered_position < 0 {
        offered_position =
            pong_publication.offer(buffer, Handlers::no_reserved_value_supplier_handler());
    }

    while image.position() < offered_position {
        let _ = image.poll(Some(handler), FRAGMENT_COUNT_LIMIT);
    }
}
