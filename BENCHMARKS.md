# Benchmarks: Aeron Java vs. Rusteron Rust IPC Throughput

This benchmark document compares the throughput performance of the existing Aeron `EmbeddedExclusiveIpcThroughput` benchmark in Java with the equivalent implementation in Rust using `rusteron_client`. The benchmark used for Java is `EmbeddedExclusiveIpcThroughput`, while the Rust implementation is ported to `rusteron-client/examples/embedded_exclusive_ipc_throughput.rs`.

**Warning**: Writing good benchmarks is always challenging, and many factors can influence the results. Until these benchmarks are verified by multiple people and in different environments, take these results with a pinch of salt.

## Benchmarks

### Exclusive IPC Throughput

This section details the exclusive IPC throughput benchmark, comparing Java and Rust implementations.

## Running the Benchmarks

### Java IPC Throughput Benchmark
The Java benchmark can be executed using the following `just` command:

```sh
just benchmark-java-ipc-throughput
```

### Rust IPC Throughput Benchmark
The Rust benchmark involves two steps. First, run the Aeron Media Driver for Rust, and then run the IPC throughput benchmark:

```sh
just run-aeron-media-driver-rust
just benchmark-rust-ipc-throughput
```

## Benchmark Results

### Java IPC Throughput Results
The results obtained by running the Java `EmbeddedExclusiveIpcThroughput` benchmark on an M1 MacBook are as follows:

```
Duration 1007ms - 29,084,302 messages - 930,697,664 payload bytes
Duration 1001ms - 28,269,764 messages - 904,632,448 payload bytes
Duration 1001ms - 27,716,075 messages - 886,914,400 payload bytes
Duration 1002ms - 28,217,443 messages - 902,958,176 payload bytes
Duration 1001ms - 26,952,002 messages - 862,464,064 payload bytes
Duration 1001ms - 29,162,348 messages - 933,195,136 payload bytes
Duration 1001ms - 28,432,625 messages - 909,844,000 payload bytes
Duration 1002ms - 29,374,684 messages - 939,989,888 payload bytes
Duration 1001ms - 27,872,288 messages - 891,913,216 payload bytes
```

### Rust IPC Throughput Results with Rust Media Driver
The results obtained by running the Rust port of the Java benchmark using Rusteron with the Rust media driver (implemented using C bindings in Rust) on the same M1 MacBook are as follows:

```
Throughput: 36,859,281 msgs/sec, 1,179,496,981 bytes/sec
Throughput: 37,699,149 msgs/sec, 1,206,372,773 bytes/sec
Throughput: 36,795,387 msgs/sec, 1,177,452,397 bytes/sec
Throughput: 36,535,444 msgs/sec, 1,169,134,222 bytes/sec
Throughput: 36,445,890 msgs/sec, 1,166,268,494 bytes/sec
Throughput: 36,736,759 msgs/sec, 1,175,576,282 bytes/sec
Throughput: 37,174,194 msgs/sec, 1,189,574,210 bytes/sec
Throughput: 37,490,359 msgs/sec, 1,199,691,476 bytes/sec
Throughput: 37,328,495 msgs/sec, 1,194,511,837 bytes/sec
Throughput: 35,525,506 msgs/sec, 1,136,816,205 bytes/sec
Throughput: 36,739,436 msgs/sec, 1,175,661,956 bytes/sec
Throughput: 36,449,411 msgs/sec, 1,166,381,156 bytes/sec
Throughput: 35,312,234 msgs/sec, 1,129,991,486 bytes/sec
Throughput: 37,361,933 msgs/sec, 1,195,581,866 bytes/sec
Throughput: 36,977,467 msgs/sec, 1,183,278,956 bytes/sec
Throughput: 36,397,079 msgs/sec, 1,164,706,525 bytes/sec
Throughput: 36,540,587 msgs/sec, 1,169,298,774 bytes/sec
Throughput: 36,122,391 msgs/sec, 1,155,916,511 bytes/sec
Throughput: 37,562,581 msgs/sec, 1,202,002,593 bytes/sec
Throughput: 36,723,374 msgs/sec, 1,175,147,956 bytes/sec
Throughput: 36,403,125 msgs/sec, 1,164,900,012 bytes/sec
Throughput: 36,450,019 msgs/sec, 1,166,400,612 bytes/sec
Throughput: 38,015,912 msgs/sec, 1,216,509,192 bytes/sec
Throughput: 37,447,615 msgs/sec, 1,198,323,687 bytes/sec
Throughput: 37,126,740 msgs/sec, 1,188,055,668 bytes/sec
```

### Rust IPC Throughput Results with Java Media Driver
The results obtained by running the Rust port of the Java benchmark using Rusteron connected to the Java media driver are as follows:

```
Throughput: 34,371,321 msgs/sec, 1,099,882,267 bytes/sec
Throughput: 36,009,246 msgs/sec, 1,152,295,888 bytes/sec
Throughput: 34,798,099 msgs/sec, 1,113,539,166 bytes/sec
Throughput: 35,104,141 msgs/sec, 1,123,332,511 bytes/sec
Throughput: 34,219,767 msgs/sec, 1,095,032,558 bytes/sec
Throughput: 33,570,332 msgs/sec, 1,074,250,618 bytes/sec
Throughput: 34,357,081 msgs/sec, 1,099,426,587 bytes/sec
Throughput: 35,308,391 msgs/sec, 1,129,868,511 bytes/sec
Throughput: 34,503,901 msgs/sec, 1,104,124,848 bytes/sec
Throughput: 34,056,613 msgs/sec, 1,089,811,617 bytes/sec
Throughput: 36,392,474 msgs/sec, 1,164,559,166 bytes/sec
Throughput: 35,717,277 msgs/sec, 1,142,952,865 bytes/sec
Throughput: 35,266,763 msgs/sec, 1,128,536,429 bytes/sec
Throughput: 35,042,202 msgs/sec, 1,121,350,469 bytes/sec
Throughput: 35,904,945 msgs/sec, 1,148,958,241 bytes/sec
Throughput: 34,771,485 msgs/sec, 1,112,687,523 bytes/sec
Throughput: 33,368,872 msgs/sec, 1,067,803,916 bytes/sec
Throughput: 33,951,504 msgs/sec, 1,086,448,141 bytes/sec
Throughput: 34,073,490 msgs/sec, 1,090,351,673 bytes/sec
Throughput: 34,825,446 msgs/sec, 1,114,414,268 bytes/sec
Throughput: 34,024,351 msgs/sec, 1,088,779,219 bytes/sec
```

### Ping Pong Benchmark

This section details the Ping Pong benchmark, which is based on the Aeron `EmbeddedPingPong` sample. The benchmark involves a warm-up phase of 100,000 messages followed by the actual benchmark of 10,000,000 messages, with a message size length of 32 bytes. Regular publishers were used, not exclusive publishers. The channels were `aeron:udp?endpoint=localhost:40123` and `aeron:udp?endpoint=localhost:40124`.

#### Rust Ping Pong Results with Rust Media Driver
The results of the Rust benchmark using the Rust media driver (implemented using C bindings in Rust) are as follows:

```
PING: pong publisher aeron:udp?endpoint=localhost:20124 1003
PING: ping subscriber aeron:udp?endpoint=localhost:20123 1002
PONG: ping publisher aeron:udp?endpoint=localhost:20123 1002
PONG: pong subscriber aeron:udp?endpoint=localhost:20124 1003

Histogram of RTT latencies in micros:
# of samples: 10000000
min: 8.496
50th percentile: 32.047
99th percentile: 103.871
99.9th percentile: 145.791
max: 32309.247
avg: 40.506064342799824
```

#### Rust Ping Pong Results with Java Media Driver
The results of the Rust benchmark with the Java media driver are as follows:

```
PING: pong publisher aeron:udp?endpoint=localhost:20124 1003
PING: ping subscriber aeron:udp?endpoint=localhost:20123 1002
PONG: ping publisher aeron:udp?endpoint=localhost:20123 1002
PONG: pong subscriber aeron:udp?endpoint=localhost:20124 1003
message length 32 bytes

Histogram of RTT latencies in micros:
# of samples: 10000000
min: 8.416
50th percentile: 34.015
99th percentile: 102.719
99.9th percentile: 183.935
max: 82378.751
avg: 44.14706427200005
```

#### Java Ping Pong Results
The results of the Java Ping Pong benchmark are as follows:

```
Publishing Ping at aeron:udp?endpoint=localhost:20123 on stream id 1002
Subscribing Ping at aeron:udp?endpoint=localhost:20123 on stream id 1002
Subscribing Pong at aeron:udp?endpoint=localhost:20124 on stream id 1003
Publishing Pong at aeron:udp?endpoint=localhost:20124 on stream id 1003
Message payload length of 32 bytes
Using exclusive publications: false
Waiting for new image from Pong...
Warming up... 10 iterations of 10,000 messages
Pinging 10,000,000 messages

Histogram of RTT latencies in microseconds.
       Value     Percentile TotalCount 1/(1-Percentile)

       8.503 0.000000000000          1           1.00
      24.383 0.500000000000    5041120           2.00
      59.423 0.990625000000    9906360         106.67
     100.159 0.999023437500    9990262        1024.00
#[Mean    =       23.587, StdDeviation   =       52.323]
#[Max     =    72351.743, Total count    =     10000000]
#[Buckets =           24, SubBuckets     =         2048]
```

## Summary
From the results above, we observe that the Rust implementation consistently achieves higher throughput compared to the Java version in the Exclusive IPC Throughput benchmark:
- **Java**: Average throughput of approximately 28 million messages per second.
- **Rust with Rust Media Driver**: Average throughput of approximately 36-38 million messages per second.
- **Rust with Java Media Driver**: Average throughput of approximately 33-36 million messages per second.

The Rust implementation shows a noticeable improvement in throughput, with an approximate **30% improvement** over the Java version when using the Rust media driver. This is higher than expected, as Java low latency applications are usually around 10-20% slower compared to implementations in C++ or Rust. This could indicate that there may be a flaw in the benchmark, and the results should be interpreted with caution.

The Ping Pong benchmark, however, tells a different story. Despite the Rust implementation outperforming Java in Exclusive IPC Throughput, the Java implementation shows faster round-trip times (RTTs) in the Ping Pong benchmark:
- **Java**: Mean RTT of approximately 23.6 microseconds.
- **Rust with Rust Media Driver**: Mean RTT of approximately 40.5 microseconds.
- **Rust with Java Media Driver**: Mean RTT of approximately 44.1 microseconds.

The reasons for Rust being slower in the Ping Pong benchmark, especially considering that it was faster in the Exclusive IPC Throughput benchmark, remain unclear. This discrepancy warrants further investigation.

## Next Steps
If you have suggestions for further optimizations or would like to contribute to the Rust port (`rusteron`), feel free to open an issue or a pull request on GitHub. We're always looking for ways to push the boundaries of performance!
