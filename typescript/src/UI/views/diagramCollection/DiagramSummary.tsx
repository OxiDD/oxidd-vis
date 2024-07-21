import React, {FC} from "react";
import {DiagramState} from "../../../state/diagrams/DiagramState";
import {IconButton, Stack, useTheme} from "@fluentui/react";
import {useWatch} from "../../../watchables/react/useWatch";
import {DiagramVisualizationSummary} from "./DiagramVisualizationSummary";
import {css} from "@emotion/css";

export const DiagramSummary: FC<{diagram: DiagramState; onDelete: () => void}> = ({
    diagram,
    onDelete,
}) => {
    const theme = useTheme();
    const watch = useWatch();
    return (
        <div
            className={css({
                backgroundColor: theme.palette.neutralLighterAlt,
            })}>
            <Stack
                horizontal
                className={css({
                    backgroundColor: theme.palette.neutralLighter,
                })}>
                <Stack.Item grow className={css({padding: theme.spacing.s1})}>
                    Diagram
                </Stack.Item>
                <Stack.Item>
                    <IconButton
                        className={css({height: "100%"})}
                        iconProps={{iconName: "cancel"}}
                        // TODO: confirmation prompt
                        onClick={onDelete}
                    />
                </Stack.Item>
            </Stack>
            <Stack
                tokens={{childrenGap: theme.spacing.m}}
                style={{padding: theme.spacing.s1}}>
                {watch(diagram.visualizations).map(visualization => (
                    <Stack.Item align="stretch" key={visualization.ID}>
                        <DiagramVisualizationSummary
                            visualization={visualization}
                            onDelete={() =>
                                diagram.removeVisualization(visualization).commit()
                            }
                        />
                    </Stack.Item>
                ))}
            </Stack>
        </div>
    );
};
