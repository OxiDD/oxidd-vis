/**
 * Adds source data to a function for debug purposes
 * @param func The function to add the source to
 * @param source The source to add
 * @returns The function with source
 */
export function funcWithSource<T extends (...s: any[]) => any>(func: T, source: any): T {
    (func as any).source = source;
    return func;
}
