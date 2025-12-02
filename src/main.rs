use anyhow::Error;
use placeholder_name::{generate, run};

fn main() -> Result<(), Error> {
    run()?;
    // generate();

    // run().unwrap();
    // let seed: u64 = rng().random();
    // let tileset = parse_tileset_xml("src/procgen/tilemaps/Rooms/tileset.xml")?;
    // // println!("{:#?}", tileset);
    // let n = 20;
    // let mut wfc = WaveFunctionCollapse::new(tileset, n, n, seed);
    // for i in 0..(n * n) {
    //     if let Err(_) = wfc.step() {
    //         break;
    //     }
    //     println!("{i}");
    // }
    // let img = wfc.render()?;
    // img.save("img.png")?;

    Ok(())
}
