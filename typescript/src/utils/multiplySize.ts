/**
 * Multiplies the given size
 * @param factor The factor  to multiple with
 * @param size The size, which is a string with a number and suffix
 */
export function multiplySize(factor: number, size: string): string {
    let [number, suffix] = size.split(/(?<![a-z])(?=[a-z])/);
    return Number(number) * factor + suffix;
}
