use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use axum::{
    extract::{Query, State},
    routing::get,
    Router,
};
use serde::Deserialize;
use tokio::{
    process::Command,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
};

/// Spawns a task that fetches the queue constantly
fn run_worker(receiver: Arc<Mutex<Receiver<String>>>, task_counter: Arc<AtomicUsize>) {
    tokio::task::spawn(async move {
        let receiver = Arc::clone(&receiver);
        // This is the same as an R repeat
        loop {
            // We get the arg
            let arg = receiver.lock().await.recv().await;
            if let Some(arg) = arg {
                let task_n = task_counter.fetch_add(1, Ordering::SeqCst);
                // We run docker
                let _ = Command::new("docker")
                    .arg("run")
                    .arg("--rm")
                    .arg("ejemplo_didier")
                    .arg(arg)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .await;
                let n_tareas = receiver.lock().await.len();
                println!("Tarea {task_n} ejecutada! Quedan {n_tareas} por ejecutarse!");
            }
        }
    });
}

#[tokio::main]
async fn main() {
    // create the channel
    let (sender, receiver) = tokio::sync::mpsc::channel::<String>(10_000);

    let receiver = Arc::new(Mutex::new(receiver));
    let task_counter = Arc::new(AtomicUsize::new(0));

    run_worker(receiver.clone(), task_counter.clone());
    run_worker(receiver.clone(), task_counter.clone());
    run_worker(receiver.clone(), task_counter.clone());
    run_worker(receiver.clone(), task_counter.clone());

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `add_task_endpoint`
        .route("/", get(add_task_endpoint))
        // Add the sender to the state
        .with_state(sender);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// The structure of the query
#[derive(Deserialize)]
struct AddTaskQuery {
    arg: String,
}

/// This is the endpoint that adds a new task to the
/// queue
async fn add_task_endpoint(
    // The state (the sender)
    State(sender): State<Sender<String>>,
    // The query
    Query(query): Query<AddTaskQuery>,
) -> &'static str {
    // We send the arg to the queue
    sender.send(query.arg).await;
    "The task has been added successfully"
}
