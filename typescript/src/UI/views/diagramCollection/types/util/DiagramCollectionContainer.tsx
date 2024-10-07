import {css} from "@emotion/css";
import {IconButton, MessageBar, Stack, useTheme} from "@fluentui/react";
import React, {FC} from "react";
import {IWatchable} from "../../../../../watchables/_types/IWatchable";
import {ICollectionStatus} from "../../../../../state/diagrams/_types/IDiagramCollection";
import {useWatch} from "../../../../../watchables/react/useWatch";

export const DiagramCollectionContainer: FC<{
    title: string;
    onDelete?: () => void;
    hideFrame?: boolean;
    status: IWatchable<ICollectionStatus>;
}> = ({title, onDelete, hideFrame, children, status}) => {
    const theme = useTheme();
    const watch = useWatch();
    const curStatus = watch(status);

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
            <div className={css({padding: theme.spacing.m})}>
                {curStatus && (
                    <MessageBar
                        messageBarType={curStatus.type}
                        styles={{root: {marginBottom: theme.spacing.m}}}>
                        {curStatus.text}
                    </MessageBar>
                )}
                {children}
            </div>
        </div>
    );
};
