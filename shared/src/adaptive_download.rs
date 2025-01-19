use futures::stream::{FuturesUnordered, StreamExt};
use log::debug;
use reqwest::Client;
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::files::DownloadEntry;
use crate::progress::ProgressBar;

const MAX_CONCURRENCY: usize = 100;
const MIN_CONCURRENCY: usize = 1;
const WINDOW_DURATION: Duration = Duration::from_secs(2);
const UPDATE_CONCURRENCY_EVERY: usize = 5;

struct DownloadRecord {
    timestamp: Instant,
    success: bool,
    latency_ms: u128,
}

struct SlidingWindow {
    window: VecDeque<DownloadRecord>,
    total: usize,
    successes: usize,
    sum_latency_successes: u128,
}

impl SlidingWindow {
    fn new() -> Self {
        Self {
            window: VecDeque::new(),
            total: 0,
            successes: 0,
            sum_latency_successes: 0,
        }
    }

    fn push(&mut self, success: bool, latency_ms: u128) {
        self.window.push_back(DownloadRecord {
            timestamp: Instant::now(),
            success,
            latency_ms,
        });

        self.total += 1;
        if success {
            self.successes += 1;
            self.sum_latency_successes += latency_ms;
        }
    }

    fn pop_expired(&mut self) {
        let now = Instant::now();
        while let Some(front) = self.window.front() {
            if now.duration_since(front.timestamp) > WINDOW_DURATION {
                let rec = self.window.pop_front().unwrap();
                self.total -= 1;
                if rec.success {
                    self.successes -= 1;
                    self.sum_latency_successes -= rec.latency_ms;
                }
            } else {
                break;
            }
        }
    }

    /// Insert the latest result, remove old ones, then compute success rate & average success latency.
    /// Success rate = successes / total
    /// Average latency = sum of successful latencies / successes (only for success).
    fn add_and_calculate(&mut self, success: bool, latency_ms: u128) -> (f64, f64) {
        self.push(success, latency_ms);
        self.pop_expired();

        if self.total == 0 {
            return (1.0, 0.0);
        }

        let success_rate = self.successes as f64 / self.total as f64;
        let avg_latency = if self.successes > 0 {
            self.sum_latency_successes as f64 / self.successes as f64
        } else {
            0.0
        };

        (success_rate, avg_latency)
    }
}

async fn download_file(client: &Client, entry: &DownloadEntry) -> anyhow::Result<u128> {
    let start = Instant::now();
    let response = client
        .get(&entry.url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let latency_ms = start.elapsed().as_millis();

    if let Some(parent_dir) = entry.path.parent() {
        tokio::fs::create_dir_all(parent_dir).await?;
    }
    let mut file = tokio::fs::File::create(&entry.path).await?;
    file.write_all(&response).await?;

    Ok(latency_ms)
}

fn is_timeout_error(e: &anyhow::Error) -> bool {
    e.downcast_ref::<reqwest::Error>()
        .is_some_and(|e| e.is_timeout())
}

/// Download a single file, returning (success, latency_ms).
/// On success, we return Ok(Some(latency_ms)).
/// If it's a timeout, we return Ok(None). If it's another error, we return Err(e).
async fn do_download(client: &Client, entry: &DownloadEntry) -> anyhow::Result<Option<u128>> {
    let latency_ms = match download_file(client, entry).await {
        Ok(r) => r,
        Err(e) => {
            // If it's a timeout, we return Ok(None), else Err
            if is_timeout_error(&e) {
                debug!("Timeout: {:?}", e);
                return Ok(None);
            } else {
                return Err(e);
            }
        }
    };

    Ok(Some(latency_ms))
}

pub async fn download_files<M>(
    download_entries: Vec<DownloadEntry>,
    progress_bar: Arc<dyn ProgressBar<M> + Send + Sync>,
) -> anyhow::Result<()> {
    progress_bar.set_length(download_entries.len() as u64);

    let client = Client::new();

    let desired_concurrency = Arc::new(AtomicUsize::new(4));

    let sliding_window = Arc::new(Mutex::new(SlidingWindow::new()));

    let mut cur_entries = download_entries;
    let mut active = FuturesUnordered::new();

    fn can_spawn_more(active_count: usize, concurrency: &Arc<AtomicUsize>) -> bool {
        active_count < concurrency.load(Ordering::SeqCst)
    }

    let spawn_if_possible = |active: &mut FuturesUnordered<_>, cur_entries: &mut Vec<_>| {
        while can_spawn_more(active.len(), &desired_concurrency) {
            if let Some(entry) = cur_entries.pop() {
                let fut = async {
                    let result = do_download(&client, &entry).await;
                    (result, entry)
                };
                active.push(fut);
            } else {
                break;
            }
        }
    };

    spawn_if_possible(&mut active, &mut cur_entries);

    let mut previous_success_time = Instant::now();

    let mut next_concurrency_update = UPDATE_CONCURRENCY_EVERY;
    loop {
        let sleep_until = previous_success_time + Duration::from_secs(30);

        let maybe_item = tokio::select! {
            item = active.next() => item,
            _ = tokio::time::sleep_until(sleep_until.into()) => {
                return Err(anyhow::anyhow!("Connection timed out"));
            }
        };
        let Some((result, entry)) = maybe_item else {
            break;
        };

        let (success, latency_ms) = match result {
            Ok(Some(latency_ms)) => {
                progress_bar.inc(1);
                previous_success_time = Instant::now();
                (true, latency_ms)
            }
            Ok(None) => {
                cur_entries.push(entry);
                (false, 0)
            }
            Err(e) => {
                return Err(e);
            }
        };

        let (success_rate, avg_latency) = {
            let mut guard = sliding_window.lock().await;
            guard.add_and_calculate(success, latency_ms)
        };

        next_concurrency_update -= 1;
        if next_concurrency_update == 0 {
            next_concurrency_update = UPDATE_CONCURRENCY_EVERY;

            let current = desired_concurrency.load(Ordering::SeqCst);
            let new_value = if success_rate > 0.9 && avg_latency < 2000.0 {
                (current + 1).min(MAX_CONCURRENCY)
            } else {
                (current - (current + 3) / 4).max(MIN_CONCURRENCY)
            };
            if new_value != current {
                desired_concurrency.store(new_value, Ordering::SeqCst);
                debug!("New concurrency: {}", new_value);
            }
        }

        spawn_if_possible(&mut active, &mut cur_entries);
    }

    Ok(())
}
