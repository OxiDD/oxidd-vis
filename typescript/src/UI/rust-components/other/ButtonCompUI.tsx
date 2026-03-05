import {ButtonComp} from "oxidd-vis-rust";
import React, {useCallback} from "react";
import {useWatch} from "../../../watchables/react/useWatch";
import {DefaultButton, IconButton, PrimaryButton} from "@fluentui/react";
import {NFC} from "../../../utils/_types/NFC";
import {IAriaRef} from "../_types/IAriaRef";

export const ButtonCompUI: NFC<{
    data: ButtonComp;
    className?: string;
    aria?: IAriaRef;
}> = ({data, className, aria}) => {
    const watch = useWatch();
    const icon = watch(data.icon);
    const text = watch(data.text);
    const primary = watch(data.primary);
    const disabled = watch(data.disabled);
    const onClick = useCallback(() => {
        data.click().commit();
    }, [data]);

    const BtnType = primary ? PrimaryButton : DefaultButton;
    if (text) {
        return (
            <BtnType
                aria-describedby={aria?.descriptionID}
                aria-labelledby={aria?.labelID}
                text={text}
                className={className}
                onClick={onClick}
                iconProps={icon ? {iconName: icon} : undefined}
                disabled={disabled}
            />
        );
    } else {
        return (
            <IconButton
                aria-describedby={aria?.descriptionID}
                aria-labelledby={aria?.labelID}
                className={className}
                aria-label={icon}
                iconProps={{iconName: icon}}
                onClick={onClick}
                disabled={disabled}
            />
        );
    }
};
