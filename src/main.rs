use warp::Filter;
use std::convert::Infallible;
use std::time::Duration;
use tokio::sync::mpsc;
use warp::ws::{Message};
use tokio::time::delay_for;

async fn sleepy() -> Result<impl warp::Reply, Infallible> {
    let seconds = 2;
    tokio::time::delay_for(Duration::from_secs(seconds)).await;
    Ok(format!("I waited {} seconds!", seconds))
}


fn main() {

    println!("start server ...");

    let route_cos = warp::path!("cos" / String / u64).map(|name, age| {
        return format!("cos --- {} --- {}", name, age + 1);
    });

    let hello = warp::path!("hello" / String)
        .map(|name| {
            return format!("Hello, {}!", name);
        });

    let routes_default = warp::any().map(|| "Index");

    let route_async = warp::path("async").and_then(sleepy);

    let route_ws = warp::path("ws")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {    
            // And then our closure will be called when it completes...
            ws.on_upgrade(|websocket| {

                use futures::{FutureExt, StreamExt};

                async move {
                    let (ws_tx, mut ws_rx) = websocket.split();

                    let (tx, rx) = mpsc::unbounded_channel();
                    tokio::task::spawn(rx.forward(ws_tx).map(|result| {
                        if let Err(e) = result {
                            eprintln!("websocket send error: {}", e);
                        }
                    }));

                    tokio::spawn(async move {
                        loop {
                            let mess = ws_rx.next().await.unwrap();
                            println!("message {:?}", mess);
                        }
                    });

                    tokio::spawn(async move {
                        //tx.send("dsadad");
                        loop {
                            
                            println!("Send ...");

                            let aa: String = "dasdas".into();
                            tx.send(Ok(Message::text(aa)));

                            println!("Czekam ...");

                            delay_for(Duration::from_secs(4)).await;
                        }
                    });
                }
            })
        });

    let route_ws_index = warp::path("ws")
        .map(|| {
            format!("ws index")
        });


    let routes = hello
        .or(route_ws)
        .or(route_ws_index)
        .or(route_cos)
        .or(route_async)
        .or(routes_default);

    for _ in 0..num_cpus::get() {
        std::thread::spawn(|| smol::run(futures::future::pending::<()>()));
    }
    smol::block_on(async {
        warp::serve(routes)
            .run(([127, 0, 0, 1], 3030))
            .await;
    });
}