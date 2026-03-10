import {BoolInputComp} from "oxidd-vis-rust";
import React, {useCallback} from "react";
import {useWatch} from "../../../watchables/react/useWatch";
import {Checkbox} from "@fluentui/react";
import {NFC} from "../../../utils/_types/NFC";
import {IAriaRef} from "../_types/IAriaRef";

export const BoolInputCompUI: NFC<{
    data: BoolInputComp;
    className?: string;
    aria?: IAriaRef;
}> = ({data, className, aria}) => {
    const watch = useWatch();
    const checked = watch(data);
    const disabled = watch(data.disabled);

    const onChange = useCallback(
        (event: React.FormEvent<HTMLElement>, checked?: boolean) => {
            if (checked !== undefined) {
                data.set(checked).commit();
            }
        },
        [data]
    );

    return (
        <Checkbox
            checked={checked}
            onChange={onChange}
            aria-describedby={aria?.descriptionID}
            aria-labelledby={aria?.labelID}
            className={className}
            disabled={disabled}
        />
    );
};
