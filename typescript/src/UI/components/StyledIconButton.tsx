import React, {FC} from "react";
import {css} from "@emotion/css";
import {IButtonProps, IconButton, IRenderFunction, useTheme} from "@fluentui/react";

export const StyledIconButton: FC<{
    tooltipId: string;
    isVisible?: boolean;
    name: string;
    icon?: string;
    onRenderIcon?: IRenderFunction<IButtonProps>;
    size: number;
    className?: string;
    onClick?: () => void;
    href?: string;
}> = ({
    tooltipId,
    isVisible = false,
    name,
    icon,
    onClick,
    size = 50,
    onRenderIcon,
    href,
    className,
}) => {
    const theme = useTheme();
    return (
        <IconButton
            aria-describedby={tooltipId}
            onRenderIcon={onRenderIcon}
            className={`${css({
                width: size,
                height: size,
                color: theme.palette.neutralPrimary,
                backgroundColor: isVisible
                    ? theme.palette.neutralLighterAlt
                    : theme.palette.neutralLight,
            })}${className ? " " + className : ""}`}
            iconProps={{iconName: icon, style: {fontSize: size * 0.5}}}
            aria-label={name}
            onClick={onClick}
            href={href}
        />
    );
};
