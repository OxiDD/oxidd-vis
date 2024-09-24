import React, {FC, useCallback} from "react";
import {DiagramState} from "../../../state/diagrams/DiagramState";
import {FontIcon, IconButton, Stack, useTheme} from "@fluentui/react";
import {css} from "@emotion/css";
import {useDragStart} from "../../../utils/useDragStart";
import {useWatch} from "../../../watchables/react/useWatch";
import {useViewManager} from "../../providers/ViewManagerContext";
import {IDiagramSection} from "../../../state/diagrams/_types/IDiagramSection";
import {DiagramVisualizationState} from "../../../state/diagrams/DiagramVisualizationState";
import {FileSource} from "../../../state/diagrams/sources/FileSource";

export const DiagramSectionSummary: FC<{
    section: IDiagramSection<unknown>;
    onDelete: () => void;
}> = ({section, onDelete}) => {
    const theme = useTheme();
    const watch = useWatch();
    const visualization = watch(section.visualization);
    const viewManager = useViewManager();
    const ref = useDragStart((position, offset) => {
        if (visualization) {
            const layout = viewManager.layoutState;
            const container = layout.allTabPanels
                .get()
                .find(c => c.tabs.some(({id}) => id == visualization.ID));
            layout
                .setDraggingData({
                    position,
                    offset,
                    removeFromPanelId: container?.id,
                    preview: <TitleBar visualization={visualization} />,
                    targetId: visualization.ID,
                })
                .commit();
        }
    });
    const clickHeader = useCallback(() => {
        if (visualization) viewManager.open(visualization).commit();
    }, [visualization]);
    const canDelete = !(section instanceof FileSource);

    return (
        <div
            ref={ref}
            style={{
                backgroundColor: theme.palette.neutralLighter,
            }}>
            <TitleBar visualization={visualization} onClick={clickHeader}>
                {canDelete && (
                    <Stack.Item>
                        <IconButton
                            className={css({height: "100%"})}
                            iconProps={{iconName: "cancel"}}
                            // TODO: confirmation prompt
                            onClick={onDelete}
                        />
                    </Stack.Item>
                )}
            </TitleBar>
        </div>
    );
};

const TitleBar: FC<{
    visualization: DiagramVisualizationState | null;
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
                {visualization ? (
                    watch(visualization.name)
                ) : (
                    <span style={{color: "#FF8888"}}>
                        <FontIcon aria-label="Error" iconName="ErrorBadge" /> Broken
                        visualization
                    </span>
                )}
            </Stack.Item>
            {children}
        </Stack>
    );
};
