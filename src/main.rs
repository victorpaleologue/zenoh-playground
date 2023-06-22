//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use async_std::task::sleep;
use clap::{App, Arg};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering::SeqCst};
use std::time::Duration;
use zenoh::config::Config;
use zenoh::prelude::r#async::*;

static TOTAL_RANDOM: AtomicI64 = AtomicI64::new(0);
static TOTAL_NOF_SAMPLES: AtomicU64 = AtomicU64::new(0);

#[async_std::main]
async fn main() {
    // Initiate logging
    env_logger::init();

    let config = parse_args();

    println!("Opening session...");
    let session = zenoh::open(config).res().await.unwrap();

    let key_expr = "test/random";
    println!("Declaring Publisher on '{key_expr}'...");
    let publisher = session.declare_publisher(key_expr).res().await.unwrap();

    let _subscriber = session
        .declare_subscriber(key_expr)
        .callback(|sample| match TryInto::<i64>::try_into(sample.value) {
            Ok(value) => match TryInto::<i32>::try_into(value) {
                Ok(value) => {
                    println!("Received: {}", value);
                    TOTAL_RANDOM.fetch_add(value as i64, SeqCst);
                    TOTAL_NOF_SAMPLES.fetch_add(1, SeqCst);
                }
                Err(e) => println!("Error, received number did not fit an i32: {}", e),
            },
            Err(e) => println!("Error: {}", e),
        })
        .res()
        .await
        .unwrap();

    let _queryable = session
        .declare_queryable("test/average")
        .callback(|query| {
            let _ = query
                .reply(Ok(Sample::new(
                    keyexpr::new("test/average").unwrap(),
                    TOTAL_RANDOM.load(SeqCst)
                        / TryInto::<i64>::try_into(TOTAL_NOF_SAMPLES.load(SeqCst)).unwrap(),
                )))
                .res(); // TODO: handle errors
        })
        .res()
        .await
        .unwrap();

    for idx in 1..u32::MAX {
        sleep(Duration::from_secs(2)).await;
        let value = rand::random::<i32>();
        println!("Publication #{}: '{}': {}", &idx, &key_expr, &value);
        publisher.put(value as i64).res().await.unwrap();
        if idx % 3 == 0 {
            let avg: i64 = session
                .get("test/average")
                .res()
                .await
                .unwrap()
                .recv_async()
                .await
                .unwrap()
                .sample
                .unwrap()
                .value
                .try_into()
                .unwrap();
            println!("Current average: {:?}", avg);
        }
    }
    _subscriber.undeclare().res().await.unwrap();
}

fn parse_args() -> Config {
    let args = App::new("zenoh pub example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE] 'The zenoh session mode (peer by default).")
                .possible_values(["peer", "client"]),
        )
        .arg(Arg::from_usage(
            "-e, --connect=[ENDPOINT]...  'Endpoints to connect to.'",
        ))
        .arg(Arg::from_usage(
            "-l, --listen=[ENDPOINT]...   'Endpoints to listen on.'",
        ))
        .arg(
            Arg::from_usage("-v, --value=[VALUE]      'The value to publish.'")
                .default_value("Pub from Rust!"),
        )
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .arg(Arg::from_usage(
            "--no-multicast-scouting 'Disable the multicast-based scouting mechanism.'",
        ))
        .get_matches();

    let mut config = if let Some(conf_file) = args.value_of("config") {
        Config::from_file(conf_file).unwrap()
    } else {
        Config::default()
    };
    if let Some(Ok(mode)) = args.value_of("mode").map(|mode| mode.parse()) {
        config.set_mode(Some(mode)).unwrap();
    }
    if let Some(values) = args.values_of("connect") {
        config
            .connect
            .endpoints
            .extend(values.map(|v| v.parse().unwrap()))
    }
    if let Some(values) = args.values_of("listen") {
        config
            .listen
            .endpoints
            .extend(values.map(|v| v.parse().unwrap()))
    }
    if args.is_present("no-multicast-scouting") {
        config.scouting.multicast.set_enabled(Some(false)).unwrap();
    }
    config
}
