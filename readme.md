# Zenoh Playground

This is an exercise about Zenoh.

## Instructions

Use Zenoh to (i) publish a random 32 bits integer every 2 seconds on "test/random",
(ii) subscribe to the published data,
and (iii) expose a queryable on "test/average" that returns the average value of the previously published numbers.

What would it change if numbers are published back-to-back, i.e. at high throughput?

> - The average will be harder to compute.
>   We may need to store a large total number to divide with another large number,
>   as we may reach the limit of precision of floats to use increments to compute the average.
> - The computation may get slower than the rate of publication.
>   Grouping the values to work with batches may help.
>   If that's not possible on the server side, doing it on the client side only would still be useful.
>   Reconsidering the API so that a lower-frequency average is published is another alternative.
>   Also, processing the data in parallel may help, using stream functions such as
>   [`for_each_concurrent`](https://rust-lang.github.io/async-book/05_streams/02_iteration_and_concurrency.html).
> - The communication may get saturated.
>   Depending on the transport layer, it would result in a loss of messages
>   or in an overwhelming delay.
> - The coroutine workers may get saturated.
>   At some point, even the publication frequency could be decreased in practice.

What does it change if there are multiple publisher, subscribers, and queryables?
> The overall number of messages potentially grows a lot, depending on the network topology.
> In more details: having many more publishers do not change much,
> since normally messages would not be exchanged unless there were subscribers.
> The same stands for queryables, where it is the rate of queries that matters.
> If you have the network relays at the right places,
> you may contain traffic to certain areas of the network.
> For instance some peer-to-peer robot exchanges within a fleet would not disturb another fleet,
> nor the cloud's traffic.

What does it change if the Zenoh applications operate in peer-to-peer or client mode?
> In peer-to-peer mode, shortcuts can be taken to spare the network relays or the cloud.
> As a consequence, it may be harder to supervise the actual traffic (depending on tools surrounding Zenoh).

In Rust, provide alternative implementations using the callback-based API and the stream-based API (with and without the use of a select).
> There is no such `select` operation in the documentation.
> Are your talking about the C function `select` ?
> It could be used on actual streams to react when data is available.
> But looking for streams in the repository only yielded
> [this example of `z_get`](https://github.com/eclipse-zenoh/zenoh/blob/1de6e2f0b4a7954407583709615a6e2260e684d4/examples/README.md?plain=1#L99).
> I might discover the stream-based API as I get some experience with Zenoh,
> but the documentation does not help at my level of understanding.

You do not need to use the memory backend.
We are expecting to use Zenoh directly from master on GitHub.

Resources:

- [Zenoh Website](https://zenoh.io/)
- [Zenoh GitHub](https://github.com/eclipse-zenoh/zenoh)
- [Zenoh Discord](https://discord.com/invite/2GJ958VuHs)

Bonus:

- A C API for embedded targets is provided by [Zenoh Pico](https://github.com/eclipse-zenoh/zenoh-pico)
