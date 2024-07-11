import {useEffect, useRef, useState} from "react";
import {IWatcher} from "../_types/IWatcher";
import {Derived} from "../Derived";
import {Observer} from "../Observer";
import {usePersistentMemo} from "../../utils/usePersistentMemo";

/**
 * A hook to obtain a watch function that automatically reloads the component when any watched dependencies change
 * @returns The watcher to be used
 */
export function useWatch(): IWatcher {
    const outWatch = useRef<IWatcher>();
    const [_, update] = useState({});
    const observer = usePersistentMemo(() => {
        const derived = new Derived<number>((watch, prev) => {
            outWatch.current = d => {
                console.log("watch");
                return watch(d);
                return d.get();
            };
            console.log("compute");
            return (prev ?? 0) + 1;
        });
        return new Observer(derived).add(() => update({}));
    }, []);
    useEffect(() => {
        console.log("add");
        return () => {
            console.log("remove");
            observer.destroy();
        };
    }, []);
    return outWatch.current!;
}
