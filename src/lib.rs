use std::thread;
use std::sync::mpsc;
use std::sync::Mutex;
use std::sync::Arc;

struct Worker {
    id: usize, 
    owned_thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let owned_thread = thread::spawn(move || loop { 
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} get a job!", id);
                    job()
                }

                Message::Terminate => {
                    println!("Worker {} is getting terminated", id);
                    break;
                }
            }
        });
        Worker { 
            id, 
            owned_thread: Some(owned_thread),
        } 
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    senders: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Result<ThreadPool, &'static str> {
        if size == 0 {
            return Err("No thread pool created");
        }
        
        let (senders, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        
        Ok(ThreadPool{ workers, senders })
    }

    pub fn execute<F>(&self, f: F) 
    where 
        F: FnOnce() + Send + 'static,
    {   
        let job = Box::new(f);
        self.senders.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("sending therminating message to all workers");

        for _ in &self.workers {
            self.senders.send(Message::Terminate).unwrap();
        }
        println!("Shutting down workers");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.owned_thread.take() {
                thread.join().unwrap()
            }
        }
    }
}
        




