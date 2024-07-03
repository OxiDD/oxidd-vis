# Watchables

Watchables a primitive datatypes, similar to observables, except optimized for usage in "derived" values. Derived values can be seen as "pure" functions that rely on foreign watchables rather than parameters. They compute their value lazily and cache their results to prevent unnecessary recomputes. They ensure that at any point in time, accessing their value is equivalent to executing the compute function directly:

```ts
const compute = watch => watch(someField) * 2;
const derived = new Derived(compute);
```

=>

```ts
compute(w => w.get()) == derived.get();
```

This equivalence holds at any point in time (except for when dirty events are dispatched), and for any compute function (which is pure except for usage of other watchables, which are all watched).

This file will be improved/extended in the future. For now, see the test files to get a rough idea for usage and primitives.
