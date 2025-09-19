use clap::Parser;
use rustpolnak::rfid_reader::{rfid_serial, Event};
use tokio::sync::broadcast;

#[derive(Parser, Debug)]
struct Args {
    /// path to serial device
    #[arg(num_args = 1.., default_values_t = vec!["stubs/dev/rfid0".to_string(), "stubs/dev/rfid1".to_string()])]
    serials: Vec<String>,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();

    let (tx, mut rx) = broadcast::channel::<Event>(128);
    for serial in args.serials {
        tokio::spawn(rfid_serial(serial, tx.clone()));
    }

    while let Ok(evt) = rx.recv().await {
        println!("{evt:?}");
    }
}
