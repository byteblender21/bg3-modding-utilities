use std::time::Instant;

use clap::Parser;
use log::info;
use bg3_unpacker::stat::parse_stat_file;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, alias = "i")]
    input_path: String,
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .try_init()
        .unwrap();

    let args = Args::parse();

    let path = args.input_path;

    let now = Instant::now();
    let stats = parse_stat_file(path.as_str());
    println!("Stats: {:?}", stats);
    let elapsed_time = now.elapsed();

    if elapsed_time.as_secs() == 0 {
        info!("Unpacking took {:?}ms", elapsed_time.as_millis());
    } else {
        info!("Unpacking took {:?}s", elapsed_time.as_secs());
    }

}