/**
 * Creates a promise that waits for the given number of milliseconds
 * @param timeMS The number of milliseconds to wait
 * @returns The promise that resolves after the given number of milliseconds
 */
export function wait(timeMS: number): Promise<void> {
    return new Promise(res => setTimeout(res, timeMS));
}
