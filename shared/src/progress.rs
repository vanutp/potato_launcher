use futures::stream::{FuturesUnordered, StreamExt};
use std::future::Future;
use std::sync::Arc;

#[derive(Clone)]
pub struct Unit {
    pub name: String,
    pub size: u64,
}

pub trait ProgressBar<M>: Sync + Send {
    fn set_message(&self, message: M);

    fn set_length(&self, length: u64);

    fn inc(&self, amount: u64);

    fn finish(&self);

    fn reset(&self) {
        self.set_length(0);
    }

    fn set_unit(&self, unit: Unit);
}

pub struct NoProgressBar;

impl<M> ProgressBar<M> for NoProgressBar {
    fn set_message(&self, _message: M) {}

    fn set_length(&self, _length: u64) {}

    fn inc(&self, _amount: u64) {}

    fn finish(&self) {}

    fn set_unit(&self, _unit: Unit) {}
}

pub fn no_progress_bar() -> Arc<dyn ProgressBar<i32> + Send + Sync> {
    Arc::new(NoProgressBar)
}

async fn create_indexed_task<T, Fut>(index: usize, task: Fut) -> (usize, anyhow::Result<T>)
where
    Fut: Future<Output = anyhow::Result<T>>,
{
    let result = task.await;
    (index, result)
}

pub async fn run_tasks_with_progress<M, T, Fut>(
    tasks: impl Iterator<Item = Fut>,
    progress_bar: Arc<dyn ProgressBar<M> + Send + Sync>,
    total_size: u64,
    max_concurrent_tasks: usize,
) -> anyhow::Result<Vec<T>>
where
    Fut: Future<Output = anyhow::Result<T>>,
{
    progress_bar.set_length(total_size);

    let mut active_tasks = FuturesUnordered::new();
    let mut task_iter = tasks.enumerate();
    let mut results = Vec::new();

    for _ in 0..max_concurrent_tasks {
        if let Some((index, task)) = task_iter.next() {
            active_tasks.push(create_indexed_task(index, task));
        }
    }

    while let Some((index, result)) = active_tasks.next().await {
        match result {
            Ok(value) => {
                progress_bar.inc(1);
                results.push((index, value));
            }
            Err(e) => {
                progress_bar.finish();
                return Err(e);
            }
        }

        if let Some((index, task)) = task_iter.next() {
            active_tasks.push(create_indexed_task(index, task));
        }
    }

    progress_bar.finish();

    results.sort_by_key(|(index, _)| *index);
    Ok(results.into_iter().map(|(_, value)| value).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::time::sleep;

    struct TestProgressBar {
        length: AtomicUsize,
        increments: AtomicUsize,
        finished: AtomicUsize,
    }

    impl TestProgressBar {
        fn new() -> Self {
            Self {
                length: AtomicUsize::new(0),
                increments: AtomicUsize::new(0),
                finished: AtomicUsize::new(0),
            }
        }

        fn get_length(&self) -> usize {
            self.length.load(Ordering::SeqCst)
        }

        fn get_increments(&self) -> usize {
            self.increments.load(Ordering::SeqCst)
        }

        fn get_finished_count(&self) -> usize {
            self.finished.load(Ordering::SeqCst)
        }
    }

    impl ProgressBar<String> for TestProgressBar {
        fn set_message(&self, _: String) {
            // do nothing
        }

        fn set_length(&self, length: u64) {
            self.length.store(length as usize, Ordering::SeqCst);
        }

        fn inc(&self, amount: u64) {
            self.increments.fetch_add(amount as usize, Ordering::SeqCst);
        }

        fn finish(&self) {
            self.finished.fetch_add(1, Ordering::SeqCst);
        }

        fn set_unit(&self, _unit: Unit) {}
    }

    #[tokio::test]
    async fn test_basic_functionality() {
        let progress_bar = Arc::new(TestProgressBar::new());

        let tasks = (0..5).map(|i| async move { anyhow::Result::<i32>::Ok(i) });

        let results = run_tasks_with_progress(tasks, progress_bar.clone(), 5, 3)
            .await
            .unwrap();

        assert_eq!(results.len(), 5);
        assert_eq!(results, vec![0, 1, 2, 3, 4]);

        assert_eq!(progress_bar.get_length(), 5);
        assert_eq!(progress_bar.get_increments(), 5);
        assert_eq!(progress_bar.get_finished_count(), 1);
    }

    #[tokio::test]
    async fn test_concurrency_limiting() {
        let active_count = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));
        let progress_bar = Arc::new(TestProgressBar::new());

        let tasks = (0..10).map(|i| {
            let active_count = active_count.clone();
            let max_concurrent = max_concurrent.clone();
            async move {
                let current = active_count.fetch_add(1, Ordering::SeqCst) + 1;

                max_concurrent.fetch_max(current, Ordering::SeqCst);

                sleep(Duration::from_millis(10)).await;

                active_count.fetch_sub(1, Ordering::SeqCst);

                anyhow::Result::<i32>::Ok(i)
            }
        });

        let results = run_tasks_with_progress(tasks, progress_bar, 10, 3)
            .await
            .unwrap();

        assert_eq!(results.len(), 10);
        assert_eq!(results, (0..10).collect::<Vec<_>>());

        assert!(max_concurrent.load(Ordering::SeqCst) <= 3);

        assert!(max_concurrent.load(Ordering::SeqCst) >= 2);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let progress_bar = Arc::new(TestProgressBar::new());

        let tasks = (0..5).map(|i| async move {
            if i == 2 {
                anyhow::Result::<i32>::Err(anyhow::Error::msg("Task failed"))
            } else {
                anyhow::Result::<i32>::Ok(i)
            }
        });

        let result = run_tasks_with_progress(tasks, progress_bar.clone(), 5, 3).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Task failed");

        assert_eq!(progress_bar.get_finished_count(), 1);
    }

    #[tokio::test]
    async fn test_empty_tasks() {
        let progress_bar = Arc::new(TestProgressBar::new());

        let tasks = (0..0).map(|i| async move { anyhow::Result::<i32>::Ok(i) });

        let results = run_tasks_with_progress(tasks, progress_bar.clone(), 0, 3)
            .await
            .unwrap();

        assert_eq!(results.len(), 0);
        assert_eq!(progress_bar.get_length(), 0);
        assert_eq!(progress_bar.get_increments(), 0);
        assert_eq!(progress_bar.get_finished_count(), 1);
    }

    #[tokio::test]
    async fn test_single_task() {
        let progress_bar = Arc::new(TestProgressBar::new());

        let tasks = std::iter::once(async { anyhow::Result::<i32>::Ok(42) });

        let results = run_tasks_with_progress(tasks, progress_bar.clone(), 1, 3)
            .await
            .unwrap();

        assert_eq!(results, vec![42]);
        assert_eq!(progress_bar.get_length(), 1);
        assert_eq!(progress_bar.get_increments(), 1);
        assert_eq!(progress_bar.get_finished_count(), 1);
    }

    #[tokio::test]
    async fn test_order_preservation_with_different_completion_times() {
        let progress_bar = Arc::new(TestProgressBar::new());

        let tasks = (0..5).map(|i| async move {
            let delay_ms = 50 - (i as u64 * 10);
            sleep(Duration::from_millis(delay_ms)).await;
            anyhow::Result::<i32>::Ok(i)
        });

        let results = run_tasks_with_progress(tasks, progress_bar.clone(), 5, 3)
            .await
            .unwrap();

        assert_eq!(results, vec![0, 1, 2, 3, 4]);
        assert_eq!(progress_bar.get_length(), 5);
        assert_eq!(progress_bar.get_increments(), 5);
        assert_eq!(progress_bar.get_finished_count(), 1);
    }
}
