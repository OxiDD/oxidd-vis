import React, {FC, useCallback} from "react";
import {ButtonConfig} from "../../../state/configuration/types/ButtonConfig";
import {DefaultButton} from "@fluentui/react";
import {useWatch} from "../../../watchables/react/useWatch";

export const ButtonConfigComp: FC<{value: ButtonConfig}> = ({value}) => {
    const watch = useWatch();
    const label = watch(value.label);
    const onClick = useCallback(() => {
        value.press();
    }, []);
    return <DefaultButton text={label} onClick={onClick} />;
};
