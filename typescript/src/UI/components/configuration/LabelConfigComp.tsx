import React, {FC} from "react";
import {Label, Stack, useTheme} from "@fluentui/react";
import {useWatch} from "../../../watchables/react/useWatch";
import {LabelConfig, LabelStyle} from "../../../state/configuration/types/LabelConfig";
import {IConfigObjectType} from "../../../state/configuration/_types/IConfigObjectType";
import {css} from "@emotion/css";

export const LabelConfigComp: FC<{
    value: LabelConfig;
    ChildComp: FC<{value: IConfigObjectType}>;
}> = ({value, ChildComp}) => {
    const watch = useWatch();
    const text = watch(value.label);
    const type = watch(value.style);
    const theme = useTheme();
    if (type == LabelStyle.Category) {
        return (
            <>
                <Label className={css({marginBottom: 10, fontSize: 20})}>{text}</Label>
                <ChildComp value={watch(value.value)} />
            </>
        );
    } else if (type == LabelStyle.Above) {
        return (
            <>
                <Label>{text}</Label>
                <ChildComp value={watch(value.value)} />
            </>
        );
    } else {
        return (
            <Stack
                horizontal
                tokens={{childrenGap: theme.spacing.s1}}
                className={css({">:nth-child(2)": {flexGrow: 1}})}>
                <Label>{text}</Label>
                <ChildComp value={watch(value.value)} />
            </Stack>
        );
    }
};
