extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate h2;
extern crate http;
extern crate tokio;

#[macro_use]
extern crate tokio_trace;
extern crate tokio_trace_log;
extern crate tokio_trace_futures;

use h2::server;

use bytes::*;
use futures::*;
use http::*;

use tokio::net::TcpListener;

use tokio_trace_futures::WithSubscriber;

pub fn main() {
    let _ = env_logger::try_init();
    let subscriber = tokio_trace_log::TraceLogger::builder()
        .with_span_entry(true)
        .with_span_exits(true)
        .with_span_closes(true)
        .with_parent_fields(true)
        .finish();
    let subscriber = tokio_trace::Dispatch::new(subscriber);

    tokio_trace::dispatcher::with_default(subscriber.clone(), || {
        let subscriber2 = subscriber.clone();
        let listener = TcpListener::bind(&"127.0.0.1:5928".parse().unwrap()).unwrap();

        info!("listening on {:?}", listener.local_addr());

        let server = listener.incoming().for_each(move |socket| {
            // let socket = io_dump::Dump::to_stdout(socket);

            let connection = server::handshake(socket)
                .and_then(|conn| {
                    println!("H2 connection bound");

                    conn.for_each(|(request, mut respond)| {
                        info!("GOT request: {:?}", request);

                        let response = Response::builder().status(StatusCode::OK).body(()).unwrap();

                        let mut send = match respond.send_response(response, false) {
                            Ok(send) => send,
                            Err(e) => {
                                error!(" error respond; err={:?}", e);
                                return Ok(());
                            }
                        };

                        println!(">>>> sending data");
                        if let Err(e) = send.send_data(Bytes::from_static(b"hello world"), true) {
                            info!("  -> err={:?}", e);
                        }

                        Ok(())
                    })
                })
                .and_then(|_| {
                    info!("~~~~~~~~~~~~~~~~~~~~~~~~~~~ H2 connection CLOSE !!!!!! ~~~~~~~~~~~");
                    Ok(())
                })
                .then(|res| {
                    if let Err(e) = res {
                        error!("  -> err={:?}", e);
                    }

                    Ok(())
                })
                .with_subscriber(subscriber2.clone());

            tokio::spawn(Box::new(connection));
            Ok(())
        })
        .map_err(|e| error!("accept error: {}", e))
        .with_subscriber(subscriber);

        tokio::run(server);
    });
}
