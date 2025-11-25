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
    - [learn-wgpu](https://sotrh.github.io/learn-wgpu/) - used to get the initial boilerplate
- [Wave Function Collapse](https://github.com/mxgmn/WaveFunctionCollapse) - a nifty algorithm for procedurally generating patterns that are locally similar to a seed pattern 