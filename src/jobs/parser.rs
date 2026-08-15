use crate::bot::Notifier;
use crate::config::Config;
use crate::db::Storage;
use crate::services::FreelancehuntClient;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{error, info};

pub fn spawn_parser_job(
    config: Arc<Config>,
    fh_client: FreelancehuntClient,
    storage: Storage,
    notifier: Notifier,
) {
    tokio::spawn(async move {
        let interval_mins = config.parse_interval_minutes.max(1);
        info!("Starting project parser job (first check in 10s, interval: {} mins)...", interval_mins);

        // Initial delay of 10s before first run
        sleep(Duration::from_secs(10)).await;

        let mut timer = interval(Duration::from_secs(interval_mins * 60));
        // Reset timer tick after initial sleep
        timer.tick().await;

        loop {
            info!("Running periodic project parse...");
            match parse_new_projects(&config, &fh_client, &storage, &notifier).await {
                Ok((sent, total)) => {
                    info!("Parser run completed: sent {}/{} new projects", sent, total);
                }
                Err(e) => {
                    error!("Error during periodic project parse: {}", e);
                }
            }

            // Wait for next interval tick
            timer.tick().await;
        }
    });
}

pub fn spawn_cleanup_job(storage: Storage) {
    tokio::spawn(async move {
        let mut timer = interval(Duration::from_secs(86400));
        loop {
            timer.tick().await;
            info!("Running daily cleanup job...");
            if let Err(e) = storage.cleanup_old(30).await {
                error!("Cleanup job error: {}", e);
            }
        }
    });
}

async fn parse_new_projects(
    config: &Config,
    fh_client: &FreelancehuntClient,
    storage: &Storage,
    notifier: &Notifier,
) -> Result<(usize, usize)> {
    let projects = fh_client.get_projects(&config.target_skill_ids, 1).await?;
    if projects.is_empty() {
        return Ok((0, 0));
    }

    let mut new_projects = Vec::new();
    for p in projects {
        if !storage.is_seen(p.id).await? {
            new_projects.push(p);
        }
    }

    if new_projects.is_empty() {
        return Ok((0, 0));
    }

    let new_ids: Vec<i64> = new_projects.iter().map(|p| p.id).collect();
    storage.mark_seen_batch(&new_ids).await?;

    let mut sent_count = 0;
    for p in &new_projects {
        match notifier.send_project(p).await {
            Ok(true) => sent_count += 1,
            _ => {}
        }
    }

    Ok((sent_count, new_projects.len()))
}
