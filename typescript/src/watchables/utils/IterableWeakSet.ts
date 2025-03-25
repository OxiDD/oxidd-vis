export class IterableWeakSet<T extends WeakKey> {
    protected items = new WeakMap<T, WeakRef<T>>();
    protected refs = new Set<WeakRef<T>>();
    protected finalization = new FinalizationRegistry<WeakRef<T>>(ref => {
        this.refs.delete(ref);
    });

    /**
     * Appends a new value to the end of the WeakSet.
     */
    public add(value: T): this {
        if (this.has(value)) return this;

        const ref = new WeakRef(value);
        this.items.set(value, ref);
        this.refs.add(ref);
        this.finalization.register(value, ref);
        return this;
    }
    /**
     * Removes the specified element from the WeakSet.
     * @returns Returns true if the element existed and has been removed, or false if the element does not exist.
     */
    public delete(value: T): boolean {
        const ref = this.items.get(value);
        if (!ref) return false;
        this.items.delete(value);
        this.refs.delete(ref);
        this.finalization.unregister(value);
        return true;
    }

    /**
     * @returns a boolean indicating whether a value exists in the WeakSet or not.
     */
    public has(value: T): boolean {
        return !!this.items.get(value);
    }

    /**
     * @returns the number of (unique) elements in Set.
     */
    public get size() {
        return this.refs.size;
    }

    public clear(): void {
        for (const val of this) this.delete(val);
    }

    public *[Symbol.iterator]() {
        for (const ref of this.refs) {
            const value = ref.deref();
            if (value !== undefined) yield value;
        }
    }
}
