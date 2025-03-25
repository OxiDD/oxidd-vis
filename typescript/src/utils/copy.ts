/**
 * Copies the given text to the clipboard
 * @param text The text to be copied
 * @param promptFallback Whether to fallback to a system prompt if copy fails
 * @returns Whether the copy was successful (older browsers may not support this)
 */
export function copy(text: string, promptFallback: boolean = true): boolean {
    try {
        navigator.clipboard.writeText(text);
        return true;
    } catch (e) {
        const textarea = document.createElement("textarea");
        textarea.textContent = text;
        textarea.style.position = "fixed"; // Prevent scrolling to bottom of page in Microsoft Edge.
        document.body.appendChild(textarea);
        textarea.select();
        try {
            return document.execCommand("copy"); // Security exception may be thrown by some browsers.
        } catch (ex) {
            if (!promptFallback) return false;
            console.warn("Copy to clipboard failed.", ex);
            prompt("Copy to clipboard: Ctrl+C, Enter", text);
            return true;
        } finally {
            document.body.removeChild(textarea);
        }
    }
}
