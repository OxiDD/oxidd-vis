import {css} from "@emotion/css";
import {IconButton, Stack, useTheme} from "@fluentui/react";
import React, {FC} from "react";

export const DiagramCollectionContainer: FC<{
    title: string;
    onDelete?: () => void;
    hideFrame?: boolean;
}> = ({title, onDelete, hideFrame, children}) => {
    const theme = useTheme();
    if (hideFrame) return <>{children}</>;

    return (
        <div
            className={css({
                backgroundColor: theme.palette.neutralQuaternaryAlt,
            })}>
            <Stack
                horizontal
                className={css({
                    backgroundColor: theme.palette.neutralLighter,
                })}>
                <Stack.Item grow className={css({padding: theme.spacing.s1})}>
                    {title}
                </Stack.Item>
                <Stack.Item>
                    {onDelete && (
                        <IconButton
                            className={css({height: "100%"})}
                            iconProps={{iconName: "cancel"}}
                            // TODO: confirmation prompt
                            onClick={onDelete}
                        />
                    )}
                </Stack.Item>
            </Stack>
            <div className={css({padding: theme.spacing.m})}>{children}</div>
        </div>
    );
};
