import React from "react";
import {TextComp} from "oxidd-vis-rust";
import {NFC} from "../../../utils/_types/NFC";
import {useWatch} from "../../../watchables/react/useWatch";
import {IAriaRef} from "../_types/IAriaRef";

export const TextCompUI: NFC<{data: TextComp; className?: string; aria?: IAriaRef}> = ({
    data,
    className,
    aria,
}) => {
    const watch = useWatch();
    const text = watch(data.text);

    if (watch(data.is_title)) {
        return (
            <h2
                className={className}
                style={{pointerEvents: "all"}}
                aria-describedby={aria?.descriptionID}
                aria-labelledby={aria?.labelID}>
                {text}
            </h2>
        );
    }
    return (
        <span style={{pointerEvents: "all"}} className={className}>
            {text}
        </span>
    );
};
