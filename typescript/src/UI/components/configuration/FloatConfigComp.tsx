import React, {
    FC,
    FormEvent,
    KeyboardEvent,
    SyntheticEvent,
    useCallback,
    useEffect,
    useRef,
} from "react";
import {IntConfig} from "../../../state/configuration/types/IntConfig";
import {SpinButton} from "@fluentui/react";
import {useWatch} from "../../../watchables/react/useWatch";
import {StyledSpinButton} from "../StyledSpinButton";
import {FloatConfig} from "../../../state/configuration/types/FloatConfig";

export const FloatConfigComp: FC<{value: FloatConfig}> = ({value}) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        const el = ref.current;
        if (el) {
            const listener = (e: globalThis.KeyboardEvent) => {
                if (e.key == "Enter") {
                    (e.target as HTMLElement).blur();
                }
            };
            el.addEventListener("keydown", listener);
            return () => el.removeEventListener("keydown", listener);
        }
    }, []);
    const watch = useWatch();
    const onChange = useCallback(
        (e: unknown, v?: string) => {
            if (v == null) return;
            let val = Number(v);
            if (isNaN(val)) return;
            let multiple = value.multiple.get();
            if (multiple != undefined) val = Math.round(val / multiple) * multiple;
            value.set(val).commit();
        },
        [value]
    );
    return (
        <StyledSpinButton
            ref={ref}
            value={watch(value) + ""}
            min={watch(value.min)}
            max={watch(value.max)}
            onChange={onChange}
            step={watch(value.multiple)}
            incrementButtonAriaLabel="Increase value by 1"
            decrementButtonAriaLabel="Decrease value by 1"
        />
    );
};
