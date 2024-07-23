import React, {FC, useCallback} from "react";
import {DiagramState} from "../../../state/diagrams/DiagramState";
import {IconButton, Stack, useTheme} from "@fluentui/react";
import {DiagramVisualizationState} from "../../../state/diagrams/DiagramVisualizationState";
import {css} from "@emotion/css";
import {useDragStart} from "../../../utils/useDragStart";
import {useWatch} from "../../../watchables/react/useWatch";
import {useAppState} from "../../providers/AppStateContext";

export const DiagramVisualizationSummary: FC<{
    visualization: DiagramVisualizationState;
    onDelete: () => void;
}> = ({visualization, onDelete}) => {
    const theme = useTheme();
    const app = useAppState();
    const ref = useDragStart((position, offset) => {
        app.views.layoutState
            .setDraggingData({
                position,
                offset,
                preview: <TitleBar visualization={visualization} />,
                targetId: visualization.ID,
            })
            .commit();
    });
    const clickHeader = useCallback(() => {
        app.open(visualization).commit();
    }, []);

    return (
        <div
            ref={ref}
            style={{
                backgroundColor: theme.palette.neutralLighter,
            }}>
            <TitleBar visualization={visualization} onClick={clickHeader}>
                <Stack.Item>
                    <IconButton
                        className={css({height: "100%"})}
                        iconProps={{iconName: "cancel"}}
                        // TODO: confirmation prompt
                        onClick={onDelete}
                    />
                </Stack.Item>
            </TitleBar>
        </div>
    );
};

const TitleBar: FC<{
    visualization: DiagramVisualizationState;
    onClick?: () => void;
}> = ({visualization, children, onClick}) => {
    const theme = useTheme();
    const watch = useWatch();
    return (
        <Stack
            horizontal
            className={css({
                backgroundColor: theme.palette.neutralLighter,
            })}>
            <Stack.Item
                onClick={onClick}
                grow
                className={css({
                    padding: theme.spacing.s1,
                    userSelect: "none",
                    cursor: "pointer",
                })}>
                {watch(visualization.name)}
            </Stack.Item>
            {children}
        </Stack>
    );
};
