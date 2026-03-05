import {StringInput, StringInputComp} from "oxidd-vis-rust";
import React, {KeyboardEventHandler, useCallback, useEffect, useState} from "react";
import {useWatch} from "../../../watchables/react/useWatch";
import {TextField} from "@fluentui/react";
import {NFC} from "../../../utils/_types/NFC";
import {useDerived} from "../../../watchables/react/useDerived";
import {IAriaRef} from "../_types/IAriaRef";

export const StringInputCompUI: NFC<{
    data: StringInputComp;
    editData?: StringInputComp;
    className?: string;
    aria?: IAriaRef;
}> = ({data, editData = data, className, aria}) => {
    const watch = useWatch();
    const text = watch(data.data);
    const [curText, setCurText] = useState(text);
    const lateSubmit = watch(data.late_submit);
    useEffect(() => {
        if (!lateSubmit) return;
        setCurText(text);
    }, [text, lateSubmit]);
    const onChange = useCallback(
        (data, newValue: string) => {
            if (lateSubmit) {
                setCurText(newValue);
            } else {
                editData.data.set(newValue).commit();
            }
        },
        [editData.data, lateSubmit]
    );
    const submit = () => {
        if (!lateSubmit) return;
        editData.data.set(curText).commit();
    };
    const onKeydown: KeyboardEventHandler<unknown> = data => {
        if (data.key != "Enter") return;
        if (multiline && !data.ctrlKey) return;
        submit();
    };

    const multiline = watch(data.multiline);
    const resizable = watch(data.multiline_resizable);
    const rows = watch(
        useDerived(
            watch => {
                const dynamic = watch(data.multiline_dynamic);
                const min = watch(data.multiline_min);
                const max = watch(data.multiline_max);
                if (!dynamic) return min ?? max;

                let rows = watch(data.data).split("\n").length;
                if (min) rows = Math.max(min, rows);
                if (max) rows = Math.min(max, rows);
                return rows;
            },
            [data]
        )
    );

    const disabled = watch(data.disabled);
    const readonly = watch(data.readonly);
    return (
        <TextField
            value={lateSubmit ? curText : text}
            onChange={onChange}
            aria-describedby={aria?.descriptionID}
            aria-labelledby={aria?.labelID}
            style={{pointerEvents: "all"}}
            className={className}
            multiline={multiline}
            styles={{fieldGroup: rows ? {minHeight: 0} : undefined}}
            rows={rows}
            onKeyDown={onKeydown}
            resizable={resizable}
            readOnly={readonly}
            onBlur={submit}
            disabled={disabled}
        />
    );
};
