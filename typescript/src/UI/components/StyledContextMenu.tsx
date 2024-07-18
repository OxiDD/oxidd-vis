import React, {FC} from "react";
import {
    ContextualMenu,
    IContextualMenuProps,
    mergeStyleSets,
    useTheme,
} from "@fluentui/react";

export const StyledContextMenu: FC<IContextualMenuProps> = props => {
    const theme = useTheme();
    const style: IContextualMenuProps["styles"] = {
        list: {
            backgroundColor: theme.palette.neutralLighterAlt,
        },
    };
    return <ContextualMenu {...props} styles={mergeStyleSets(style, props.styles)} />;
};
