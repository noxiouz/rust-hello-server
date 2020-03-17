use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: Option<mpsc::SyncSender<Job>>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
                loop {
                    let job: Job = match receiver.lock().unwrap().recv() {
                        Ok(job) => job,
                        Err(e) => { println!("no new messages {}", e); return }
                    };
                    println!("Worler {} got a job", id);
                    job(); 
                    thread::sleep(Duration::from_secs(10));
                }
            }
        );
        Worker{
            id,
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::sync_channel(0);

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            println!("{}", id);
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool{
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
        where 
            F: FnOnce() + Send + 'static
    {   
        let job = Box::new(f);
        if let Some(ref sender) = self.sender {
            if let Err(e) = sender.try_send(job) {
                println!("Queeue is full ? {}", e);
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.sender.take();
        for w in &mut self.workers {
            if let Some(t) = w.thread.take() {
                t.join().unwrap();
            }
        }
    }
}