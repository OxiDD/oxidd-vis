/**
 * Intersperses the given element between all consecutive elements
 * @param values The values to insert the value into
 * @param insert The value to be inserted
 * The list with values interspersed
 */
export function intersperse<T, K>(values: T[], insert: K): (T & K)[] {
    return values.flatMap(v => [insert, v]).slice(1) as (T & K)[];
}

/**
 * Intersperses the given element between all consecutive elements
 * @param values The values to insert the value into
 * @param insert The value to be inserted
 * The list with values interspersed
 */
export function intersperseDynamic<T, K>(
    values: T[],
    insert: (index: number, before: T) => K
): (T & K)[] {
    return values.flatMap((v, i) => [insert(i, v), v]).slice(1) as (T & K)[];
}
