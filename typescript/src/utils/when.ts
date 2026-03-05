/**
 * Used to map a value if defined
 * @param value The value to be mapped
 * @param f The mapping function
 */
export function when<T, R>(value: T | undefined, f: (v: T) => R): R | undefined {
    return value === undefined ? undefined : f(value);
}
