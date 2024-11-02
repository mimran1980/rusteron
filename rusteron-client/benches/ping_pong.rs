use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusteron_client::*;
use rusteron_media_driver::{AeronDriver, AeronDriverContext};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const PING_STREAM_ID: i32 = 1002;
const PONG_STREAM_ID: i32 = 1003;
const PING_CHANNEL: &str = "aeron:udp?endpoint=localhost:20123";
const PONG_CHANNEL: &str = "aeron:udp?endpoint=localhost:20124";
const MESSAGE_LENGTH: usize = 32;
const FRAGMENT_COUNT_LIMIT: usize = 10;

fn criterion_benchmark(c: &mut Criterion) {
    let ctx = AeronDriverContext::new().unwrap();
    let dir = ctx.get_dir().to_string().leak();
    let dir2 = ctx.get_dir().to_string().leak();
    let (stop, handle) = AeronDriver::launch_embedded(ctx.clone(), false);
    let stop2 = stop.clone();
    let _pong_thread = thread::Builder::new()
        .name("pong".to_string())
        .spawn(move || run_pong(stop2, dir).unwrap())
        .unwrap();

    let context = AeronContext::new().unwrap();
    println!("idle sleep {}", context.get_idle_sleep_duration_ns());
    context.set_idle_sleep_duration_ns(0).unwrap();
    context.set_dir(dir2).unwrap();
    let aeron = Aeron::new(&context).unwrap();
    aeron.start().unwrap();

    let pong_publication = aeron
        .async_add_publication(PONG_CHANNEL, PONG_STREAM_ID)
        .unwrap()
        .poll_blocking(Duration::from_secs(5))
        .unwrap();
    let ping_subscription = aeron
        .async_add_subscription(
            PING_CHANNEL,
            PING_STREAM_ID,
            Handlers::no_available_image_handler(),
            Handlers::no_unavailable_image_handler(),
        )
        .unwrap()
        .poll_blocking(Duration::from_secs(4))
        .unwrap();

    println!("PING: pong publisher {} {}", PONG_CHANNEL, PONG_STREAM_ID);
    println!("PING: ping subscriber {} {}", PING_CHANNEL, PING_STREAM_ID);

    // Set up the ping thread
    let mut buffer = vec![0u8; MESSAGE_LENGTH];

    let mut handler = Handler::leak(PingRoundTripHandler {});

    c.bench_function("ping_benchmark", |b| {
        b.iter(|| {
            record_rtt(
                &pong_publication,
                &ping_subscription,
                &mut buffer,
                &mut handler,
            );
        });
    });

    stop.store(false, Ordering::SeqCst);
}

fn run_pong(stop: Arc<AtomicBool>, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let context = AeronContext::new()?;
    context.set_dir(dir)?;
    context.set_idle_sleep_duration_ns(0)?;
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
    while !stop.load(Ordering::Acquire) {
        let _ = pong_subscription.poll(Some(&handler), FRAGMENT_COUNT_LIMIT);
    }
    println!("Shutting down pong thread");
    Ok(())
}

pub struct PingRoundTripHandler {}

impl AeronFragmentHandlerCallback for PingRoundTripHandler {
    fn handle_aeron_fragment_handler(&mut self, buffer: &[u8], _header: AeronHeader) {
        let time = read_i64(buffer);
        let rtt = Aeron::nano_clock() - time;
        black_box(rtt);
        debug_assert!(rtt >= 0);
    }
}

fn read_i64(buffer: &[u8]) -> i64 {
    i64::from_le_bytes(
        buffer[0..8]
            .try_into()
            .expect("Slice with incorrect length"),
    )
}

#[inline]
fn record_rtt(
    pong_publication: &AeronPublication,
    ping_subscription: &AeronSubscription,
    buffer: &mut Vec<u8>,
    handler: &mut Handler<PingRoundTripHandler>,
) {
    let now = Aeron::nano_clock();
    write_i64(buffer, &now);
    while pong_publication.offer(buffer, Handlers::no_reserved_value_supplier_handler()) < 0 {}

    while ping_subscription
        .poll(Some(handler), FRAGMENT_COUNT_LIMIT)
        .unwrap_or_default()
        == 0
    {}
}

fn write_i64(buffer: &mut Vec<u8>, now: &i64) {
    buffer[0..8].copy_from_slice(&now.to_le_bytes());
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
