# SVG TileServer
![Build Status](https://github.com/GaspardCulis/svg-tileserver/actions/workflows/rust.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/svg-tileserver.svg)](https://crates.io/crates/svg-tileserver)

A simple, memory-safe 🔥 and blazingly fast 🚀 Leaflet/MapLibre compatile tile-server that serves PNG tiles rasterized from an SVG image. This can be useful when needing to render highly complex and detailed SVGs.

Built using [actix_web](https://actix.rs/) and crates from the [resvg project](https://github.com/RazrFalcon/resvg).

## Usage

### Server side

```
Usage: svg_tileserver [OPTIONS] <SVG_PATH>

Arguments:
  <SVG_PATH>  The path of the SVG that should be served

Options:
  -t, --tile-size <TILE_SIZE>        The size in pixels of a PNG tile [default: 256]
  -p, --port <PORT>                  The port to start the server on [default: 8080]
  -b, --bind-address <BIND_ADDRESS>  The size in pixels of a PNG tile [default: 127.0.0.1]
  -h, --help                         Print help
  -V, --version                      Print version
```

### Client side

```js
import L from "leaflet";

const map = new L.Map("#map", {
  crs: L.CRS.Simple, 
  center: [0, 0],
  zoom: 0
});

L.tileLayer('https://localhost:8080/{z}/{x}/{y}.png', {
    maxZoom: 19
}).addTo(map);
```
