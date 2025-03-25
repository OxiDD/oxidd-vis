import React, {FC, useCallback, useMemo} from "react";
import {ButtonConfig} from "../../../state/configuration/types/ButtonConfig";
import {DefaultButton, DirectionalHint, IconButton} from "@fluentui/react";
import {useWatch} from "../../../watchables/react/useWatch";
import {StyledTooltipHost} from "../StyledToolTipHost";
import {usePersistentMemo} from "../../../utils/usePersistentMemo";
import {v4 as uuid} from "uuid";

export const ButtonConfigComp: FC<{value: ButtonConfig}> = ({value}) => {
    const id = usePersistentMemo(() => uuid(), []);
    const watch = useWatch();
    const label = watch(value.label);
    const icon = watch(value.icon);
    const onClick = useCallback(() => {
        value.press();
    }, []);

    if (icon) {
        const iconEl = (
            <IconButton
                aria-describedby={id}
                iconProps={{iconName: icon}}
                aria-label={icon}
                onClick={onClick}
            />
        );
        if (label) {
            return (
                <StyledTooltipHost
                    directionalHint={DirectionalHint.leftCenter}
                    id={id}
                    content={label}>
                    {iconEl}
                </StyledTooltipHost>
            );
        } else {
            return iconEl;
        }
    } else {
        return <DefaultButton text={label} onClick={onClick} />;
    }
};
