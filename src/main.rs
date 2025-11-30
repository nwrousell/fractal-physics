use crate::procgen::{WaveFunctionCollapse, parse_tileset_xml};
use anyhow::Error;

mod procgen;

fn main() -> Result<(), Error> {
    // run().unwrap();
    let tileset = parse_tileset_xml("src/procgen/tilemaps/knots/knots.xml")?;
    let mut wfc = WaveFunctionCollapse::new(tileset, 50, 50, 17);
    for _ in 0..(50 * 50) {
        wfc.step()?;
    }
    let img = wfc.render()?;
    img.save("img.png")?;

    Ok(())
}
