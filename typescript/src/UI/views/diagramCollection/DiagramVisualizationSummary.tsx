import React, {FC} from "react";
import {DiagramState} from "../../../state/diagrams/DiagramState";
import {IconButton, Stack, useTheme} from "@fluentui/react";
import {DiagramVisualizationState} from "../../../state/diagrams/DiagramVisualizationState";
import {css} from "@emotion/css";
import {useDragStart} from "../../../utils/useDragStart";
import {useLayoutState} from "../../providers/LayoutStateContext";
import {useWatch} from "../../../watchables/react/useWatch";

export const DiagramVisualizationSummary: FC<{
    visualization: DiagramVisualizationState;
    onDelete: () => void;
}> = ({visualization, onDelete}) => {
    const theme = useTheme();
    const layout = useLayoutState();
    const ref = useDragStart((position, offset) => {
        layout
            .setDraggingData({
                position,
                offset,
                preview: <TitleBar visualization={visualization} />,
                targetId: visualization.ID,
            })
            .commit();
    });

    return (
        <div
            ref={ref}
            style={{
                backgroundColor: theme.palette.neutralLighter,
            }}>
            <TitleBar visualization={visualization}>
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
}> = ({visualization, children}) => {
    const theme = useTheme();
    const watch = useWatch();
    return (
        <Stack
            horizontal
            className={css({
                backgroundColor: theme.palette.neutralLighter,
            })}>
            <Stack.Item
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
