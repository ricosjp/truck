# Ad-hoc viewer by `truck-js`

## How to run

### Prepare

```bash
cargo install wasm-pack basic-http-server
```

### Running script

```bash
wasm-pack build --target web
cp examples/bootstrap.js examples/index.html examples/script.js pkg
cd pkg
basic-http-server -a 127.0.0.1:8080
```

## How to use

- Dragging the mouse rotates the model.
- To change the shape, select the shape file (json) modeled by truck-modeling from the button below the canvas.

## Pre-build page

[![adhoc-viewer](https://img.shields.io/badge/Adhoc-Viewer-lightgrey)](https://ricos.pages.ritc.jp/truck/truck/adhoc-viewer)
