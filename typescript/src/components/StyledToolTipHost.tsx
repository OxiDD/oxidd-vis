import React, {FC} from "react";
import {
    ContextualMenu,
    ICalloutProps,
    IContextualMenuProps,
    ITooltipHostProps,
    TooltipDelay,
    TooltipHost,
    mergeStyleSets,
    useTheme,
} from "@fluentui/react";

export const StyledTooltipHost: FC<ITooltipHostProps> = props => {
    const theme = useTheme();
    const color = theme.palette.neutralLight;
    const style: ICalloutProps["styles"] = {
        beakCurtain: {background: color},
        beak: {background: color},
        calloutMain: {background: color},
    };
    return (
        <TooltipHost
            delay={TooltipDelay.long}
            {...props}
            calloutProps={{
                ...props.calloutProps,
                styles: mergeStyleSets(style, props.calloutProps?.styles),
            }}
        />
    );
};
