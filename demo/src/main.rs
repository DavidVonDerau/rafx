// There's a decent amount of code that's just for example and isn't called
#![allow(dead_code)]

use structopt::StructOpt;

pub fn logging_init() {
    #[cfg(not(debug_assertions))]
    let log_level = log::LevelFilter::Info;
    #[cfg(debug_assertions)]
    let log_level = log::LevelFilter::Info;

    // Setup logging
    env_logger::Builder::from_default_env()
        .default_format_timestamp_nanos(true)
        .filter_module(
            "renderer_assets::resources::descriptor_sets",
            log::LevelFilter::Info,
        )
        .filter_module("renderer_shell_vulkan::device", log::LevelFilter::Debug)
        .filter_module("renderer_nodes", log::LevelFilter::Info)
        .filter_module("renderer_visibility", log::LevelFilter::Info)
        .filter_module("renderer_assets::graph", log::LevelFilter::Info)
        // .filter_module(
        //     "renderer_assets::resources::command_buffers",
        //     log::LevelFilter::Trace,
        // )
        .filter_level(log_level)
        // .format(|buf, record| { //TODO: Get a frame count in here
        //     writeln!(buf,
        //              "{} [{}] - {}",
        //              chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
        //              record.level(),
        //              record.args()
        //     )
        // })
        .init();
}

fn main() {
    logging_init();

    let args = demo::DemoArgs::from_args();

    demo::run(&args);
}
