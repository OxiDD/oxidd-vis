import {useEffect, useRef, useState} from "react";
import {IWatcher} from "../_types/IWatcher";
import {Observer} from "../Observer";
import {usePersistentMemo} from "../../utils/usePersistentMemo";
import {PassiveDerived} from "../PassiveDerived";

/**
 * A hook to obtain a watch function that automatically reloads the component when any watched dependencies change
 * @returns The watcher to be used
 */
export function useWatch(): IWatcher {
    const observer = useRef<Observer<unknown>>();
    if (observer.current) observer.current.destroy(); // Dispose data from the last render

    const outWatch = useRef<IWatcher>();
    // We use passive derived, so we don't retain a lot of unnecessary listeners until garbage collection kicks in
    const derived = new PassiveDerived<number>((watch, prev) => {
        outWatch.current = watch;
        return (prev ?? 0) + 1;
    });
    const [_, update] = useState(1);
    observer.current = new Observer(derived).add(() => update(x => x + 1));

    useEffect(() => () => observer.current?.destroy(), []);
    return outWatch.current!;
}
