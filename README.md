# Graphics Final Project

## To Run

### As a webapge
1. Install `wasm-pack` from [here](https://drager.github.io/wasm-pack/)
2. `wasm-pack build --target web`
3. `python -m http.server 8080`
4. open `localhost:8080` in your browser

### As a standalone window
1. `cargo run`

## Resources
- [`wgpu`](https://github.com/gfx-rs/wgpu)
    - [learn-wgpu](https://sotrh.github.io/learn-wgpu/)
- [Wave Function Collapse](https://github.com/mxgmn/WaveFunctionCollapse) - a nifty algorithm for procedurally generating patterns that are locally similar to a seed pattern 
    - [More explanation](https://trasevol.dog/2017/09/01/di19/)
    - [Visual explanation](https://www.dropbox.com/scl/fi/tlrf4iw34dxc83vy562ao/Knots-breakdown.png?rlkey=3ys3lsis59jz94dr5seb3bcho&e=1&dl=0) of symmetry + neighbor encoding
- [Postprocessing WebGPU tutorial](https://webgpufundamentals.org/webgpu/lessons/webgpu-post-processing.html)
