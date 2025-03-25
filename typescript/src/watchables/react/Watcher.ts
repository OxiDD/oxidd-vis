import {FC, ReactNode} from "react";
import {IWatcher} from "../_types/IWatcher";
import {useWatch} from "./useWatch";

/**
 * A watcher component that takes a callback that uses the watcher to compute the children of this component
 * @param props The props of this component
 * @returns The react node
 */
export const Watcher: FC<{children: (watcher: IWatcher) => JSX.Element}> = ({
    children,
}) => {
    const watch = useWatch();
    return children(watch);
};
