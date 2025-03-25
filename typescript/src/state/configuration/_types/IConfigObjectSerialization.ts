export type IConfigObjectSerialization<V> = {
    value: V;
    children: IConfigObjectSerialization<unknown>[];
};
