use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use clap::Parser;
use env_logger::Target;
use log::LevelFilter;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(long, short = 'v')]
    verbose: bool,

    #[clap(long)]
    sit: bool,

    #[clap(long)]
    stand: bool,

    /// When specified, displays desk height to the console, if move, sit or stand is not specified,
    /// then just monitor only log on manual move triggered by the user.
    #[clap(long, short = 'm')]
    monitor: bool,

    #[clap(long = "move", short = 't')]
    move_to: Option<u8>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli_arguments: Args = Args::parse();

    let env = env_logger::Env::default().filter_or("RUST_LOG", "info");

    if cli_arguments.verbose {
        env_logger::Builder::from_env(env)
            .filter_level(LevelFilter::Debug)
            .target(Target::Stdout)
            .init();
    } else {
        env_logger::Builder::from_env(env)
            .target(Target::Stdout)
            .init();
    }

    log::debug!("input arguments {:?}", cli_arguments);

    let personal_desk_address = "C2:6D:5B:C4:17:12";
    let desk = idasen_desk::Desk::new(personal_desk_address).await?;

    log::info!("connected to desk: {:?}", desk.name);

    // handle the case in which the device target amount was specified. // we allow this being a
    // whole number, e.g 74, which will be later converted into a float value.
    if let Some(target_value) = cli_arguments.move_to {
        desk.move_to_target((target_value as f32) / 100.0).await?;
        return Ok(());
    }

    log::trace!("{}", desk.to_string());

    let current_desk_height = desk.get_height().await?;
    log::debug!("starting desk position {:?}", current_desk_height);

    // if the user has specified sit or stand.
    if cli_arguments.stand {
        desk.move_to_target(1.12).await?;
        return Ok(());
    } else if cli_arguments.sit {
        desk.move_to_target(0.74).await?;
        return Ok(());
    }

    if cli_arguments.monitor && (!cli_arguments.stand && !cli_arguments.sit) {
        let desk_height = Arc::new(Mutex::new(0.0));
        let mut previous_desk_height = 0.0;

        let _ = desk.monitor_height_notification_stream(desk_height.clone()).await?;

        loop {
            let height = *desk_height.lock().unwrap();
            if height != previous_desk_height {
                log::info!("height: {height}");
                previous_desk_height= height;
            }

            // small sleep between checks to not to spam CPU cycles.
            sleep(Duration::from_millis(50))
        }
    }

    Ok(())
}
