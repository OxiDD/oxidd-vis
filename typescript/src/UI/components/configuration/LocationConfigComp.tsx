import React, {FC, useCallback} from "react";
import {useTheme} from "@fluentui/react";
import {useWatch} from "../../../watchables/react/useWatch";
import {IConfigObjectType} from "../../../state/configuration/_types/IConfigObjectType";
import {css} from "@emotion/css";
import {LocationConfig} from "../../../state/configuration/types/LocationConfig";

export const LocationConfigComp: FC<{
    value: LocationConfig;
    ChildComp: FC<{value: IConfigObjectType}>;
}> = ({value, ChildComp}) => {
    const theme = useTheme();
    const watch = useWatch();
    const x = watch(value.horizontal);
    const y = watch(value.vertical);
    const p = watch(value.padding);
    const s = theme.spacing.m;

    // To prevent drag interactions with the visualization when clicking inside this container
    const preventDrag = useCallback((e: React.MouseEvent) => {
        e.stopPropagation();
    }, []);
    return (
        <div
            className={css({
                position: "absolute",
                left: `calc(${p}*${s} + ${x} * (100% - 2*${p}*${s}) )`,
                top: `calc(${p}*${s} + ${y} * (100% - 2*${p}*${s}) )`,
            })}>
            <div
                onMouseDown={preventDrag}
                className={css({transform: `translate(${-x * 100}%, ${-y * 100}%)`})}>
                <ChildComp value={watch(value.value)} />
            </div>
        </div>
    );
};
