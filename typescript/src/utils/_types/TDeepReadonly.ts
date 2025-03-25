export type TDeepReadonly<T> = Readonly<{
    [K in keyof T]: // Is it a primitive? Then make it readonly
    T[K] extends number | string | symbol
        ? Readonly<T[K]>
        : // Is it an array of items? Then make the array readonly and the item as well
        T[K] extends Array<infer A>
        ? Readonly<Array<TDeepReadonly<A>>>
        : // It is some other object, make it readonly as well
          TDeepReadonly<T[K]>;
}>;
