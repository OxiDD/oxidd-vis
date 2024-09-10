/**
 * Converts a binary sequence to a string
 * @param bytes The bytes to encode
 * @returns The string representation
 */
export function binaryToString(bytes: Uint8Array): string {
    return String.fromCharCode.apply(null, bytes);
}

/**
 * Converts a string to a binary sequence
 * @param data The data to be decoded
 * @returns The binary sequence
 */
export function stringToBinary(data: string): Uint8Array {
    const array = new Uint8Array(data.length);
    for (let i = 0; i < data.length; i++) {
        array[i] = data.charCodeAt(i);
    }
    return array;
}
