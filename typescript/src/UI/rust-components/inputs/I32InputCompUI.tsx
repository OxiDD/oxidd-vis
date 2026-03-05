import {I32InputComp} from "oxidd-vis-rust";
import React, {useCallback, useEffect, useState} from "react";
import {useWatch} from "../../../watchables/react/useWatch";
import {SpinButton, TextField} from "@fluentui/react";
import {NFC} from "../../../utils/_types/NFC";
import {IAriaRef} from "../_types/IAriaRef";

export const I32InputCompUI: NFC<{
    data: I32InputComp;
    editData?: I32InputComp;
    className?: string;
    aria?: IAriaRef;
}> = ({data, editData = data, className, aria}) => {
    const watch = useWatch();
    const valData = data.data;
    const min = watch(valData.min);
    const max = watch(valData.max);
    const clamped = watch(valData.clamped);
    const round_step = watch(data.step_round);
    const disabled = watch(data.disabled);
    const is_stepper = false; //step_size != undefined;

    const onChange = useCallback(
        (event: unknown, newValue?: string) => {
            if (newValue === undefined) return;
            const parsed = parseInt(newValue, 10);
            if (!isNaN(parsed)) {
                editData.data.input.set(parsed).commit();
            }
            if (!is_stepper) {
                setTextfieldValue(valData.clamped.get().toString());
            }
        },
        [editData.data.input, is_stepper]
    );

    // Textfield functions
    const [textfieldValue, setTextfieldValue] = useState(clamped.toString());
    useEffect(() => {
        if (is_stepper) return;
        setTextfieldValue(clamped.toString());
    }, [clamped, is_stepper]);
    const onTextfieldChange = useCallback(
        (event: unknown, newValue?: string) => {
            if (newValue === undefined) return;
            setTextfieldValue(newValue);
        },
        [editData.data.input]
    );
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
            const current = valData.clamped.get();
            if (isNaN(current)) return;

            const stepSize = data.step_size.get() ?? 1;
            let next = current + direction * stepSize;
            if (round_step) {
                next = Math.round(next / stepSize) * stepSize;
            }
            editData.data.input.set(next).commit();
        },
        [data, editData, valData, is_stepper]
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
                min={min}
                max={max}
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
            min={min}
            max={max}
            disabled={disabled}
        />
    );
};
