use anyhow::Error;
use clap::{Parser, Subcommand};
use placeholder_name::{render_rects_to_file, render_scene_to_file, run_interactive, run_wfc};

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
        /// Optional PNG file to load from
        #[arg(long)]
        from: Option<String>,
    },
    /// Render rectangles to file
    RenderRects {
        /// Output file path
        path: String,
        /// Optional PNG file to load from
        #[arg(long)]
        from: Option<String>,
    },
    /// Render scene
    RenderScene {
        /// Output file path
        path: String,
        /// Optional PNG file to load from
        #[arg(long)]
        from: Option<String>,
        /// Output image width (default: 1920)
        #[arg(long, default_value = "1920")]
        width: u32,
        /// Output image height (default: 1080)
        #[arg(long, default_value = "1080")]
        height: u32,
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
        /// Path to tileset XML file (default: src/procgen/tilemaps/Rooms/tileset.xml)
        #[arg(long)]
        tileset: Option<String>,
    },
}

fn main() -> Result<(), Error> {
    let args = Cli::parse();

    match args.command {
        Commands::Interactive { from } => {
            run_interactive(from.as_deref())?;
        }
        Commands::RenderRects { path, from } => {
            render_rects_to_file(from.as_ref().map(|s| s.as_str()), &path)?;
        }
        Commands::RenderScene {
            path,
            from,
            width,
            height,
        } => {
            render_scene_to_file(from.as_deref(), &path, width, height)?;
        }
        Commands::Wfc {
            path,
            seed,
            n,
            tileset,
        } => {
            run_wfc(seed, n, &path, tileset.as_deref())?;
        }
    }

    Ok(())
}
