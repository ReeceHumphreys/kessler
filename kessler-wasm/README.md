# kessler-wasm

WebAssembly bindings for [Kessler](../README.md). Build with [wasm-pack](https://rustwasm.github.io/wasm-pack/):

```shell
wasm-pack build kessler-wasm
```

```js
import { SatKind, collision, explosion } from "kessler-wasm";

// All inputs and outputs are SI (metres, kilograms, metres per second).
const debris = explosion(839.0, SatKind.RocketBody, 0.1, 42); // seed is optional
debris.mass;    // Float32Array, one entry per fragment
debris.deltaV;  // Float32Array, flattened [x0, y0, z0, x1, ...]

collision(500.0, SatKind.Spacecraft, 5.0, SatKind.Spacecraft, 9500.0, 0.1);
```
