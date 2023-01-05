use std::env;


mod tail;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let path = &args[1];

    let mut tail = tail::Tail::new(path).await;

    tail.start();

    loop {
        let lines = tail.get_lines().await;
        println!("{:?}", lines);
    }
}
