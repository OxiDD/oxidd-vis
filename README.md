# OxiDD-viz

A decision diagram visualization tool. This tool is a work in progress, with many more possible features coming in the future, as well as code refactors that simplify adding support for new decision diagram types.

## Development

To run this application, you both have to build the rust-code, and run a web-development server. See the instructions below

### Rust

Install:

- Rust + cargo: https://www.rust-lang.org/tools/install (tested version: 1.79.0)
- Wasm-pack: https://rustwasm.github.io/wasm-pack/installer/ (tested version: 0.12.1)
<!-- - wasm2map: https://crates.io/crates/wasm2map (tested version 0.1.0) -->

Build code using (run from inside the rust directory):

```
wasm-pack build --dev
```

This includes more readable error messages on panic, for better runtime performance, build using:

```
wasm-pack build --release
```

### TypeScript

Install:

- https://nodejs.org/en (tested version: 18.15.0)

Install code dependencies (run from inside the typescript directory):

```
npm install
```

Note, do not use yarn for this install!! Yarn does not create a dynamic link.

Start development server (after having built the rust code):

```
npm run start
```

View the website at: http://localhost:3000/
