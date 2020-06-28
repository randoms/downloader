use std::sync::{mpsc, Arc, Mutex};
use log::{info};

type Task = Box<dyn FnOnce() + Send + 'static>;
pub struct ThreadPool {
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(num: u64) -> ThreadPool {
        let mut threadpool = ThreadPool {
            workers: Vec::with_capacity(num as usize),
        };
        for _ in 0..num {
            threadpool.workers.push(Worker::new());
        }
        return threadpool;
    }

    pub fn add_task<F>(&mut self, task: F) where F: FnOnce() + Send + 'static {
        let free_workers: Vec<&Worker> = self.workers.iter().filter(|w|  WorkerStatus::Free == w.get_worker_status()).collect();
        if free_workers.is_empty() {
            self.workers.sort_by(|a, b| a.get_padding_task().cmp(&b.get_padding_task()));
            self.workers[0].add_task(Box::new(task)).expect("add task failed");
            return;
        }
        free_workers[0].add_task(Box::new(task)).expect("add task failed");
    }
}

#[derive(Eq, PartialEq)]
enum WorkerStatus {
    Free,
    Working,
}
struct Worker {
    status: Arc<Mutex<WorkerStatus>>,
    // rx: mpsc::Receiver<Task>,
    tx: mpsc::Sender<Task>,
    padding_task_count: Arc<Mutex<u64>>,
}

impl Worker {
    pub fn new() -> Worker {
        let (tx, rx) = mpsc::channel();
        let worker_status = WorkerStatus::Free;
        let worker = Worker {
            status: Arc::new(Mutex::new(worker_status)),
            tx: tx,
            padding_task_count: Arc::new(Mutex::new(0)),
        };
        let status1= Arc::clone(&worker.status);
        let padding_task_count1 = Arc::clone(&worker.padding_task_count);
        std::thread::spawn(move || {
            {
                let mut status = status1.lock().unwrap();
                *status = WorkerStatus::Working;
            }
            for task in rx {
                task();
                {
                    let mut status = status1.lock().unwrap();
                    *status = WorkerStatus::Free;
                }
                {
                    let mut padding_task_count = padding_task_count1.lock().unwrap();
                    *padding_task_count -= 1;
                }
            }
        });
        return worker;
    }

    pub fn get_worker_status(&self) -> WorkerStatus {
        if *self.status.lock().unwrap() == WorkerStatus::Working {
            return WorkerStatus::Working;
        }
        return WorkerStatus::Free;
    }

    pub fn add_task(&self, task: Task) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut padding_task_count = self.padding_task_count.lock().unwrap();
            *padding_task_count += 1;
        }
        self.tx.send(task).expect("Send task failed");
        Ok(())
    }

    pub fn get_padding_task(&self) -> u64 {
        return *self.padding_task_count.lock().unwrap();
    }
}
