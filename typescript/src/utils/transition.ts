/**
 * Performs a transition by calling the given callback repeatedly with a new fractional progress value
 * @param perform The function to perform
 * @param durationMS  The number of milliseconds that the transition should last
 * @returns A function that can be used to cancel the transition
 */
export function transition(
    perform: (per: number) => void,
    durationMS: number
): {cancel: () => void} {
    let start = Date.now();
    const intervalID = setInterval(() => {
        let delta = Date.now() - start;
        let per = Math.min(1.0, delta / durationMS);
        perform(per);
        if (per >= 1) stop();
    }, 0);
    const stop = () => clearInterval(intervalID);
    return {cancel: stop};
}
