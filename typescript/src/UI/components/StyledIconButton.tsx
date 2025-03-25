import React, {FC} from "react";
import {css} from "@emotion/css";
import {IconButton, useTheme} from "@fluentui/react";

export const StyledIconButton: FC<{
    tooltipId: string;
    isVisible?: boolean;
    name: string;
    icon: string;
    size: number;
    onClick: () => void;
}> = ({tooltipId, isVisible = false, name, icon, onClick, size = 50}) => {
    const theme = useTheme();
    return (
        <IconButton
            aria-describedby={tooltipId}
            className={css({
                width: size,
                height: size,
                color: theme.palette.neutralPrimary,
                backgroundColor: isVisible
                    ? theme.palette.neutralLighterAlt
                    : theme.palette.neutralLight,
            })}
            iconProps={{iconName: icon, style: {fontSize: size * 0.5}}}
            aria-label={name}
            onClick={onClick}
        />
    );
};
