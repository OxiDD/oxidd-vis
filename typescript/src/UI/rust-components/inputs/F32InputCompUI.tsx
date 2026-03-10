import {F32InputComp} from "oxidd-vis-rust";
import React, {useCallback, useEffect, useState} from "react";
import {useWatch} from "../../../watchables/react/useWatch";
import {SpinButton, TextField} from "@fluentui/react";
import {NFC} from "../../../utils/_types/NFC";
import {IAriaRef} from "../_types/IAriaRef";

export const F32InputCompUI: NFC<{
    data: F32InputComp;
    className?: string;
    aria?: IAriaRef;
}> = ({data, className, aria}) => {
    const watch = useWatch();
    const clamped = watch(data);
    const round_step = watch(data.step_round);
    const disabled = watch(data.disabled);
    const is_stepper = watch(data.step_size) != undefined;

    const onChange = useCallback(
        (event: unknown, newValue?: string) => {
            if (newValue === undefined) return;
            const parsed = parseFloat(newValue);
            if (!isNaN(parsed)) {
                data.set(parsed).commit();
            }
            if (!is_stepper) {
                setTextfieldValue(data.get().toString());
            }
        },
        [data, is_stepper]
    );

    // Textfield functions
    const [textfieldValue, setTextfieldValue] = useState(clamped.toString());
    useEffect(() => {
        if (is_stepper) return;
        setTextfieldValue(clamped.toString());
    }, [clamped, is_stepper]);
    const onTextfieldChange = useCallback((event: unknown, newValue?: string) => {
        if (newValue === undefined) return;
        setTextfieldValue(newValue);
    }, []);
    const handleKeyDown = useCallback(
        (event: React.KeyboardEvent<HTMLInputElement>) => {
            if (event.key === "Enter")
                onChange(event, (event.target as HTMLInputElement).value);
        },
        [onChange]
    );

    // Stepper functions
    const step = useCallback(
        direction => {
            const current = data.get();
            if (isNaN(current)) return;

            const stepSize = data.step_size.get() ?? 1;
            let next = current + direction * stepSize;
            if (round_step) {
                next = Math.round(next / stepSize) * stepSize;
            }
            data.set(next).commit();
        },
        [data, is_stepper]
    );
    const onDecrement = useCallback(() => step(-1), [step]);
    const onIncrement = useCallback(() => step(1), [step]);

    if (is_stepper) {
        return (
            <SpinButton
                value={clamped.toString()}
                onChange={onChange}
                aria-describedby={aria?.descriptionID}
                aria-labelledby={aria?.labelID}
                className={className}
                onIncrement={onIncrement}
                onDecrement={onDecrement}
                disabled={disabled}
            />
        );
    }
    return (
        <TextField
            value={textfieldValue}
            onBlur={event => onChange(event, (event.target as HTMLInputElement).value)}
            onKeyDown={handleKeyDown}
            aria-describedby={aria?.descriptionID}
            aria-labelledby={aria?.labelID}
            className={className}
            onChange={onTextfieldChange}
            disabled={disabled}
        />
    );
};
