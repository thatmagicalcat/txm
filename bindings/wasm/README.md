<div align="center">
  <h1>TXM.wasm</h1>
  <p>WebAssembly / JavaScript bindings for TXM.</p>
</div>

### Build

```
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
cd bindings/wasm
wasm-pack build --target nodejs
```

### Example (Node.js)

```js
const { render } = require('./pkg');
console.log(render('E = mc^2'));
console.log(render('\\int_0^\\infty e^{-x^2}\\,dx = \\frac{\\sqrt{\\pi}}{2}'));
```

## License
- Apache License, Version 2.0 / MIT license
