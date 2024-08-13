mod channels;
mod config;
mod notifier;
mod opensearch_watcher;

use std::{collections::HashMap, env, path::PathBuf, process, sync::Arc, time::Duration};

use crate::config::Config;
use notifier::Notifier;
use std::error::Error;

use channels::WebhookChannel;
use config::DSLRule;
use opensearch_watcher::OpenSearchWatcher;
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [conf.toml]", args[0]);
        process::exit(1);
    }

    let conf_path = PathBuf::from(&args[1]);

    let conf = match Config::try_from(conf_path.as_path()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing config: {:?}", e);
            process::exit(1);
        }
    };

    let mut webhook_channels: HashMap<String, WebhookChannel> = HashMap::new();

    for (channel_name, channel) in conf.channels.iter() {
        let webhook_channel = match WebhookChannel::try_from(channel) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "Error parsing channel {:?} as a WebhookChannel: {:?}",
                    channel, e
                );
                process::exit(1);
            }
        };

        webhook_channels.insert(channel_name.clone(), webhook_channel);
    }

    println!("{:?}", &conf);

    let (tx, rx) = channel(1024);

    let rules: Vec<Arc<DSLRule>> = conf
        .rules
        .values()
        .map(|r| Arc::new(DSLRule::try_from(r).expect("Failed to parse rule")))
        .collect();

    let indexes: Vec<&str> = conf.general.indexes.iter().map(|s| s.as_str()).collect();

    let watcher = OpenSearchWatcher::new(
        &conf.general.opensearch_url,
        &indexes,
        Duration::from_secs(conf.general.watch_interval.unwrap_or(15)),
        tx,
    )
    .await?;

    let notifier = Notifier::new(webhook_channels);

    let _notifier_h = tokio::spawn(async move { notifier.run(rx).await });

    if let Err(e) = watcher.watch(rules.as_slice()).await {
        eprintln!("Got error on search: {:?}", e);
    }

    Ok(())
}
