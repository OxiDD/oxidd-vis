# BDD-viz

OxiDD visualization application

## Development

To run this application, you both have to build the rust-code, and run a web-development server. See the instructions below

### Rust

Install:

- Rust + cargo: https://www.rust-lang.org/tools/install (tested version: 1.79.0)
- Wasm-pack: https://rustwasm.github.io/wasm-pack/installer/ (tested version: 0.12.1)

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

Start development server (after having built the rust code):

```
npm run start
```

View the website at: http://localhost:3000/

## TODO:

- [x] text: Add text rendering
- [x] level: Add level rendering:
  - [ ] Add level collapsing layout algorithm that condenses levels when nothing happens in them
- [ ] stepping: Add BDD-algorithm stepping
- [ ] algorithms: Develop own algorithms:
  - [ ] Layout algorithm (primarily node ordering per level)
  - [ ] Node revealing algorithm for exploration
  - [ ] Node grouping algorithm to hide details:
    - [ ] Grouping conjunction chains
    - [ ] Level-wise node-grouping
    - [ ] ...
- [ ] gui: Create GUI around visualization:
  - [x] Create watchables data-structure
  - [x] Create panel based UI, modified from rascal-vis
  - [x] Create node selection system + visualization
  - [ ] Create node selection stats panel
  - [ ] Create algorithm-stepping controls
  - [ ] Create algorithm-application UI to select an algorithm to apply to some given diagram (nodes)
  - [ ] Create settings:
    - [ ] Show/hide terminals (true and false independently controllable)
    - [ ] Duplicate terminals (" ")
    - [ ] Label edges
    - [ ] Label nodes
    - [ ] Hide levels
    - [ ] Animation duration
- [ ] OxiDD: Integrate OxiDD properly
- [ ] source: Create source selection method, allowing for:
  - [ ] Inputting a BDD in text form
  - [ ] Inputting from a logic formula or set specification
  - [ ] Syncing with a server, allowing OxiDD to communicate diagrams
