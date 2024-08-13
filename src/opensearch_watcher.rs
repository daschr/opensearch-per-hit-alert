use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::DSLRule;
use chrono::{DateTime, Utc};
use opensearch::{
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    http::Url,
    Error as OpenSearchError, OpenSearch,
};
use tokio::sync::mpsc::Sender;

pub struct OpenSearchWatcher {
    client: OpenSearch,
    interval: Duration,
    indexes: Vec<String>,
    tx: Sender<(Arc<DSLRule>, Value)>,
}

struct WatchContext {
    last_checked_time: DateTime<Utc>,
}

impl OpenSearchWatcher {
    pub async fn new(
        url: &str,
        indexes: &[&str],
        interval: Duration,
        tx: Sender<(Arc<DSLRule>, Value)>,
    ) -> Result<Self, OpenSearchError> {
        let trans = TransportBuilder::new(SingleNodeConnectionPool::new(Url::parse(url)?))
            .cert_validation(opensearch::cert::CertificateValidation::None)
            .build()?;

        let indexes = indexes
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        Ok(OpenSearchWatcher {
            client: OpenSearch::new(trans),
            interval,
            indexes,
            tx,
        })
    }

    pub async fn watch(&self, rules: &[Arc<DSLRule>]) -> Result<(), OpenSearchError> {
        let indexes: Vec<&str> = self.indexes.iter().map(|s| s.as_str()).collect();

        let ts = DateTime::from_timestamp(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            0,
        )
        .expect("Failed to create UTC timestamp");

        let mut ctxs: Vec<WatchContext> = (0..rules.len())
            .map(|_| WatchContext {
                last_checked_time: ts,
            })
            .collect();

        loop {
            for rule_id in 0..rules.len() {
                self.query_entries(
                    rules[rule_id].clone(),
                    &mut ctxs[rule_id],
                    indexes.as_slice(),
                )
                .await?;
            }

            tokio::time::sleep(self.interval).await;
        }
    }

    async fn query_entries(
        &self,
        rule: Arc<DSLRule>,
        ctx: &mut WatchContext,
        indexes: &[&str],
    ) -> Result<(), OpenSearchError> {
        let resp = self
            .client
            .search(opensearch::SearchParts::Index(indexes))
            .sort(&["@timestamp:desc"])
            .from(0)
            .size(100)
            .body(&rule.query)
            .send()
            .await?;

        println!("RULE: {}", rule.name);
        println!("QUERY: {}", rule.query);
        println!("TS: {}", ctx.last_checked_time);

        let json_resp = resp.json::<serde_json::Value>().await.unwrap();

        let hits = json_resp["hits"]["hits"].as_array().unwrap();

        for entry in hits {
            let e_s = &entry["_source"];
            let entry_ts = DateTime::parse_from_rfc3339(e_s["@timestamp"].as_str().unwrap())
                .expect("Failed to parse timestamp of entry")
                .to_utc();

            if entry_ts <= ctx.last_checked_time {
                break;
            }

            self.tx.send((rule.clone(), entry.clone())).await.ok();
        }

        if !hits.is_empty() {
            let newest_ts =
                DateTime::parse_from_rfc3339(hits[0]["_source"]["@timestamp"].as_str().unwrap())
                    .expect("Failed to parse timestamp of entry")
                    .to_utc();
            ctx.last_checked_time = newest_ts;
        }

        Ok(())
    }
}
