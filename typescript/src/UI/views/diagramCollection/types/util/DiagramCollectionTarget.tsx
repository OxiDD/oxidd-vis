import React, { FC, useEffect } from "react";
import { HttpDiagramCollectionTargetState } from "../../../../../state/diagrams/collections/HttpDiagramCollectionState";
import { CenteredContainer } from "../../../../components/layout/CenteredContainer";
import { useTheme } from "@fluentui/react";
import { useViewManager } from "../../../../providers/ViewManagerContext";
import { all } from "../../../../../watchables/mutator/all";
import { DiagramVisualizationState } from "../../../../../state/diagrams/DiagramVisualizationState";

export const DiagramCollectionTarget: FC<{ target: HttpDiagramCollectionTargetState }> = ({
    target,
}) => {
    const theme = useTheme();
    const viewManager = useViewManager();
    useEffect(
        () =>
            target.onDiagramOpen(diagram =>
                all(
                    diagram.sections
                        .get()
                        .map(section => section.visualization.get())
                        .filter((vis): vis is DiagramVisualizationState => !!vis)
                        .filter(vis => !viewManager.isOpen(vis).get())
                        .map(vis =>
                            viewManager.open(vis, function* (hints) {
                                yield {
                                    targetId: target.ID,
                                    targetType: "view",
                                    tabIndex: { target: target.ID, position: "after" },
                                };
                                yield* hints;
                            })
                        )
                )
            ),
        []
    );

    return (
        <CenteredContainer>
            <h2
                style={{
                    color: theme.palette.themeSecondary,
                    fontSize: 20,
                    marginTop: 10,
                }}>
                {target.host} target
            </h2>
            <p>
                Newly synchronized diagrams will automatically be opened in the same panel
                that this target is opened in. Close it to stop automatic opening of
                diagrams.
            </p>
        </CenteredContainer>
    );
};
