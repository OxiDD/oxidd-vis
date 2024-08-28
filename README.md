# BDD-viz

OxiDD visualization application

TODO:s

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
  - [ ] Create node selection system + visualization
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
