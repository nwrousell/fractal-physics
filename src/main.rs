use anyhow::Error;
use clap::{Parser, Subcommand};
use placeholder_name_lib::{
    parse_and_render_rects, render_scene_to_file, run_interactive, run_wfc,
};

#[derive(Parser)]
#[command(name = "placeholder-name")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run interactive mode
    Interactive {
        /// PNG world file
        #[arg(long, default_value = "default_world.png")]
        from: String,

        /// disables postprocessing
        #[arg(long, default_value_t = false)]
        dont_postprocess: bool,
    },
    /// Render rectangles to file
    RenderRects {
        /// Output file path
        path: String,
        /// PNG world file
        #[arg(long, default_value = "default_world.png")]
        from: String,
    },
    /// Render scene
    RenderScene {
        /// Output file path
        path: String,
        /// PNG world file
        #[arg(long, default_value = "default_world.png")]
        from: String,
        /// Output image width (default: 1920)
        #[arg(long, default_value = "1920")]
        width: u32,
        /// Output image height (default: 1080)
        #[arg(long, default_value = "1080")]
        height: u32,

        /// disables postprocessing
        #[arg(long, default_value_t = false)]
        dont_postprocess: bool,
    },
    /// Run WFC and save to file
    Wfc {
        /// Output file path
        path: String,
        /// Seed for WFC generation
        seed: u64,
        /// Width/height of WFC wave (default: 10)
        #[arg(short, long, default_value = "10")]
        n: usize,
        /// Path to tileset XML file
        #[arg(long, default_value = "src/procgen/tilemaps/Rooms/tileset.xml")]
        tileset: String,
    },
}

fn main() -> Result<(), Error> {
    let args = Cli::parse();

    match args.command {
        Commands::Interactive {
            from,
            dont_postprocess,
        } => {
            run_interactive(&from, !dont_postprocess)?;
        }
        Commands::RenderRects { path, from } => {
            parse_and_render_rects(&from, &path)?;
        }
        Commands::RenderScene {
            path,
            from,
            width,
            height,
            dont_postprocess,
        } => {
            render_scene_to_file(&from, &path, width, height, !dont_postprocess)?;
        }
        Commands::Wfc {
            path,
            seed,
            n,
            tileset,
        } => {
            run_wfc(seed, n, &path, &tileset)?;
        }
    }

    Ok(())
}
