# Rust Hyper async/sync benchmarks

I am comparing the usage of `sync` vs. `async` rust on top of async [`hyper`](https://github.com/hyperium/hyper) (i.e. incoming TCP requests and HTTP parsing is async in both cases - this is not about async hyper vs sync hyper!).

**tl;dr;**: Both approaches work. There are differences and what to use depends — as almost always — on the use case.

I've build a chatbot (not public) using async Rust. Since the ergonomics of a long and branched async chains is currently (this is expected to improve eventually) really bad and already feels unmaintainable, I've converted the chatbot into using sync APIs ontop of async hyper (by handling requests in a [`CpuPool`](https://github.com/alexcrichton/futures-rs)). I benchmarked both implementations (result is at the end of the README) and the sync one performed – suprisingly – better. However, due to different libraries doing the same (e.g. sync postgres vs, async postgres, sync postgres pool vs, async postgres pool, sync web "framework" vs async web "framework") I am not trusting those benchmarks. This is why I've tried to build a more fair comparison here. The benchmark does not simply return a hello world, instead it involves some simple JSON deserialization, a short calculation and an artifical delay, which acts as a database query replacement.

I am open to discussions about the results and also about critic and suggestions of how to improve this benchmark to reflect a real world scenario even more.

Some learnings:

- the result of a sync implementation can be tuned by the thread pool size (bigger is not always better)

| Bench | Start with | Description |
| --- | --- | --- |
| sync | `cargo run --release --bin sync` | Async [`hyper`](https://github.com/hyperium/hyper), requests are executed synchronously inside a [`CpuPool`](https://github.com/alexcrichton/futures-rs) (pool size: 32 threads) |
| async1 | `cargo run --release --bin async1` | Async [`hyper`](https://github.com/hyperium/hyper), requests are executed asynchronously |
| async2 | `cargo run --release --bin async2` | Async [`hyper`](https://github.com/hyperium/hyper), requests are executed asynchronously and JSON deserialization and processing is offloaded to a [`CpuPool`](https://github.com/alexcrichton/futures-rs) |

## Benchmark Results

The following tests what async is good at. Almost no blocking work (like long running calculations).

```bash
hey -m POST -T 'application/json' -d '[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]' -n 10000 -c 100 http://127.0.0.1:3000/
```

(Rust 1.21.0-nightly)

**2.9 GHz Intel Core i5 (2 physical cores; macOS); 20ms artifical delay**

| Bench | Pool Size | Total time in sec | Requests/sec |
| --- | --- | --- | --- |
| sync | 32 | 1.8692 | 5349.7821 |
| sync | 100 | 2.0110 | 4972.5688 |
| async1 | n/a | 2.1219 | 4712.8329 |
| async2 | 32 | 2.3795 | 4202.5000 |
| async2 | 100 | 2.5816 | 3873.6418 |

**2.9 GHz Intel Core i5 (2 physical cores; macOS); 200ms artifical delay**

| Bench | Pool Size | Total time in sec | Requests/sec |
| --- | --- | --- | --- |
| sync | 32 | 63.5640 | 157.3218 |
| sync | 100 | 20.7050 | 482.9746 |
| async1 | n/a | 20.6325 | 484.6731 |
| async2 | 32 | 20.7229 | 482.5580 |
| async2 | 100 | 20.7803 | 481.2247 |

**2.9 GHz Intel Core i5 (2 physical cores; macOS); 1000ms artifical delay**

As expected, the total time is bound to the toal amount of requests multiplied with the 1s artifical delay. Except in the `sync` example, when we have less threads in the thread pool than concurrent requests. The `async` case will probably always take `requests count * delay` seconds, regardless of the amount of concurrent requests.

| Bench | Pool Size | Total time in sec | Requests/sec |
| --- | --- | --- | --- |
| sync | 32 | 314.3733 | 31.8093 |
| sync | 100 | 100.6027 | 99.4009 |
| async1 | n/a | 100.6358 | 99.3682 |
| async2 | 32 | 100.6814 | 99.3232 |
| async2 | 100 | 101.3967 | 98.6225 |

**3.5 GHz Intel Core i5-6600K (4 physical cores; Win10); 20ms artifical delay**

| Bench | Pool Size | Total time in sec | Requests/sec |
| --- | --- | --- | --- |
| sync | 32 | 1.1350 | 8810.4345 |
| sync | 100 |  1.0538 | 9489.4406 |
| async1 | n/a | 0.8513 | 11747.2439 |
| async2 | 32 | 1.5411 | 6488.8792 |
| async2 | 100 | 0.9926 | 10074.4187 |

**3.5 GHz Intel Core i5-6600K (4 physical cores; Win10); 200ms artifical delay**

| Bench | Pool Size | Total time in sec | Requests/sec |
| --- | --- | --- | --- |
| sync | 32 | 62.8029 | 159.2283 |
| sync | 100 | 20.1590 | 496.0560 |
| async1 | n/a | 20.1549 | 496.1566 |
| async2 | 32 | 20.1605 | 496.0196 |
| async2 | 100 | 20.1667 | 495.8673 |

## Chatbot Benchmark Results

These are the results of the chatbot benchmarks. I am posting them here to show how a database connection pool size (or my bad pool implementation - not sure) effects the results:

1000 requests, 100 parallel

| Postgres Pool Size | Async (req/sec) | Sync (32 worker threads; req/sec) |
| --- | --- | --- |
| 4 | 1658.8334 | 2851.0067 |
| 10 | 1906.7896 | 2821.7548 |
| 16 | 2123.7447 | 2859.8802 |
| 20 | 2422.2993 | 2839.3450 |
| 32 | 2442.2338 | 2746.1324 |
| 100 | 2324.2620 | 2378.4929 |
