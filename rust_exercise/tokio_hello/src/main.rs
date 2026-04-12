use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let job1 = tokio::spawn(async {
        println!("Hello from the 1st spawned task!");
        sleep(Duration::from_secs(2)).await;
        println!("Goodbye from the 1st spawned task after 2 seconds!");
    });

    let job2 = tokio::spawn(async {
        println!("Hello from the 2nd spawned task!");
        sleep(Duration::from_secs(1)).await;
        println!("Goodbye from the 2nd spawned task after 1 second!");
    });

    job1.await.unwrap();
    job2.await.unwrap();
}
