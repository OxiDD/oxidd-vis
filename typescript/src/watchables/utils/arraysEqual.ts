/**
 * Checks whether two arrays are equivalent in terms of members
 * @param a
 * @param b
 * @returns Whether they arrays are equivalent
 */
export function arraysEqual<T>(a: T[], b: T[]) {
    return a.length == b.length && a.every((v, i) => v == b[i]);
}
