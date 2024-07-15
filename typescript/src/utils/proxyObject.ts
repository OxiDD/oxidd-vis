import {IField} from "../state/_types/IField";
import {Derived} from "../watchables/Derived";
import {IWatchable} from "../watchables/_types/IWatchable";
import {DerivedField} from "./DerivedField";

export const proxy = Symbol();
export type IProxyAble<T> = T & {
    /**
     * Creates a proxy of this object, proxying to the given target, with this object as a fallback
     * @param target The target to proxy to
     * @returns The created proxy
     */
    [proxy](target: IWatchable<T | undefined>): IProxyAble<T>;
};

/**
 * Creates a proxyable object
 * @param obj The object to make proxyable
 * @returns The proxyable object
 */
export function proxyObject<
    T extends {[K: string]: IProxyAble<unknown> | IField<unknown>}
>(obj: T): IProxyAble<Readonly<T>> {
    const proxyFunc = (target: IWatchable<T | undefined>) => {
        return Object.fromEntries(
            Object.keys(obj).map(key => {
                const value = obj[key];
                if (proxy in value) {
                    const newObject = value[proxy](
                        new Derived(watch => watch(target)?.[key])
                    );
                    return [key, newObject];
                } else {
                    const defaultField = value;
                    const newField = new DerivedField(
                        watch => {
                            const curTarget = watch(target);
                            if (!curTarget) return watch(defaultField);

                            const field = curTarget[key] as IField<any>;
                            if (!field) return watch(defaultField);

                            return watch(field);
                        },
                        newValue => {
                            const field = target.get()?.[key] as IField<any> | undefined;
                            if (field) return field.set(newValue);
                            return defaultField.set(newValue);
                        }
                    );
                    return [key, newField];
                }
            })
        );
    };
    return {...obj, [proxy]: proxyFunc};
}
