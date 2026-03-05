import {IWatchable} from "../_types/IWatchable";
import {Derived} from "../Derived";

/**
 * Maps a list of data, and lazily performs `map` based on new items being created
 * @param list The list of items to map
 * @param map The mapper function
 * @returns The watchable mapped output
 */
export function mapList<I, O>(
    list: IWatchable<I[]>,
    map: (data: I) => IWatchable<O>
): IWatchable<O[]> {
    const mapped = new Derived<{itemMap: Map<I, IWatchable<O>>; list: IWatchable<O>[]}>(
        (watch, oldData) => {
            let out = [];
            let itemMap = watch(list);
            let newMap = new Map();
            for (let item of itemMap) {
                let itemOut = (oldData ? oldData.itemMap.get(item) : null) ?? map(item);
                out.push(itemOut);
                newMap.set(item, itemOut);
            }

            return {itemMap: newMap, list: out};
        }
    );
    return new Derived(watch => {
        let {list} = watch(mapped);
        return list.map(item => watch(item));
    });
}
