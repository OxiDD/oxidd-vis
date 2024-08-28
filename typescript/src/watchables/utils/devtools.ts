import {IterableWeakSet} from "./IterableWeakSet";

export const inspect = Symbol("inspect");
export type ISummary = {short?: Object; long: Object};
export type IInspectable = {[inspect](): ISummary};

/** A formatter for watchables */
export const watchablesFormatter = {
    header: (object: any, config: any) => {
        if (isInspectable(object)) {
            const summary = object[inspect]();
            const out = summary.long;
            Object.setPrototypeOf(out, {
                [Symbol.toStringTag]:
                    getTypeName(object) +
                    (summary.short ? " " + format(summary.short) : ""),
            });
            return ["object", {object: out}];
        }
        return null;
    },
    hasBody: () => false,
};

/** A formatter for iterable weaksets */
export const weaksetFormatter = {
    header: (object: any) => {
        if (object instanceof IterableWeakSet) {
            const refs = (object as any).refs;
            return ["object", {object: refs}];
        }
        return null;
    },
    hasBody: () => false,
};

/**
 * Installs the devtools formatter
 */
export function installDevtools() {
    const formatters = ((window as any).devtoolsFormatters =
        (window as any).devtoolsFormatters ?? []);
    formatters.push(watchablesFormatter);
    formatters.push(weaksetFormatter);
    console.log("Watchables formatters setup, make sure custom formatters are enabled!");
}

function getTypeName(object: any): string {
    if (object && Symbol.toStringTag in object) {
        return object[Symbol.toStringTag];
    }
    return Object.getPrototypeOf(object).constructor.name;
}

function isInspectable(object: any): object is IInspectable {
    return inspect in object;
}

function format(object: Object) {
    let out = "{";
    for (const key in object) {
        out += key + ": ";
        const val = (object as any)[key];
        if (typeof val == "object") {
            out += getTypeName(val);
        } else {
            out += val;
        }
    }
    out += "}";
    return out;
}
