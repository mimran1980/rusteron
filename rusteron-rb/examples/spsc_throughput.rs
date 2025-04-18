use rusteron_rb::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

const BURST_LENGTH: usize = 1_000_000;
const MESSAGE_LENGTH: usize = 32;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let running = Arc::new(AtomicBool::new(true));

    println!("message length {}", MESSAGE_LENGTH);

    let running_ctrl_c = Arc::clone(&running);
    ctrlc::set_handler(move || {
        running_ctrl_c.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let running_publisher = Arc::clone(&running);
    let running_subscriber = Arc::clone(&running);

    let rb = Arc::new(AeronSpscRb::new_with_capacity(1024 * 1024, MESSAGE_LENGTH)?);

    let publisher_thread = {
        let rb = rb.clone();
        thread::spawn(move || {
            Publisher::new(running_publisher, rb).run();
            Ok::<_, AeronCError>(())
        })
    };

    let subscriber_thread = thread::spawn(move || {
        let mut image_rate_subscriber =
            ImageRateSubscriber::new(running_subscriber, rb, MESSAGE_LENGTH);
        image_rate_subscriber.run();
        Ok::<_, AeronCError>(())
    });

    publisher_thread
        .join()
        .expect("Publisher thread failed")
        .unwrap();
    subscriber_thread
        .join()
        .expect("Subscriber thread failed")
        .unwrap();

    Ok(())
}

struct Publisher {
    running: Arc<AtomicBool>,
    publication: Arc<AeronSpscRb>,
}

impl Publisher {
    fn new(running: Arc<AtomicBool>, publication: Arc<AeronSpscRb>) -> Self {
        Publisher {
            running,
            publication,
        }
    }

    fn run(&self) {
        let mut back_pressure_count = 0;
        let mut total_message_count = 0;
        while self.running.load(Ordering::Acquire) {
            if let Ok(mut msg) = self.publication.try_claim_slice(1, MESSAGE_LENGTH) {
                msg[0] = 1u8;
            } else {
                back_pressure_count += 1;
                if !self.running.load(Ordering::Acquire) {
                    let back_pressure_ratio =
                        back_pressure_count as f64 / total_message_count as f64;
                    println!("Publisher back pressure ratio: {:.6}", back_pressure_ratio);
                    println!("total_message_count: {total_message_count}");
                    return;
                }
            }
            total_message_count += 1;
        }

        let back_pressure_ratio = back_pressure_count as f64 / total_message_count as f64;
        println!("Publisher back pressure ratio: {:.6}", back_pressure_ratio);
        println!("total_message_count: {total_message_count}");
    }
}

struct ImageRateSubscriber {
    running: Arc<AtomicBool>,
    subscription: Arc<AeronSpscRb>,
    handler: Handler<AeronRingBufferHandlerWrapper<MsgCount>>,
    message_length: usize,
    start_time: Instant,
}

struct MsgCount {
    message_count: usize,
}

impl AeronRingBufferHandlerCallback for MsgCount {
    fn handle_aeron_rb_handler(&mut self, msg_type_id: i32, buffer: &[u8]) {
        self.message_count += 1;
        assert_eq!(msg_type_id, 1);
        assert_eq!(buffer[0], 1);
    }
}

impl ImageRateSubscriber {
    fn new(
        running: Arc<AtomicBool>,
        subscription: Arc<AeronSpscRb>,
        message_length: usize,
    ) -> Self {
        let poll_handler = AeronRingBufferHandlerWrapper::new(MsgCount { message_count: 0 });
        ImageRateSubscriber {
            running,
            subscription,
            handler: poll_handler,
            message_length,
            start_time: Instant::now(),
        }
    }

    fn run(&mut self) {
        let mut next_check = BURST_LENGTH;
        while self.running.load(Ordering::Acquire) {
            let _ = self.subscription.read_msgs(&self.handler, BURST_LENGTH);

            if self.handler.message_count >= next_check
                && self.start_time.elapsed() >= Duration::from_secs(1)
            {
                next_check += BURST_LENGTH;
                let elapsed = self.start_time.elapsed().as_secs_f64();
                let rate = self.handler.message_count as f64 / elapsed;
                let throughput = rate * self.message_length as f64;

                use num_format::{Locale, ToFormattedString};
                println!(
                    "Throughput: {} msgs/sec, {} bytes/sec",
                    (rate.round() as u64).to_formatted_string(&Locale::en),
                    (throughput.round() as u64).to_formatted_string(&Locale::en)
                );

                self.start_time = Instant::now();
                self.handler.message_count = 0;
            }
        }
    }
}
