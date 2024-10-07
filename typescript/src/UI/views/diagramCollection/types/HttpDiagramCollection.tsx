import React, {FC, useCallback} from "react";
import {useWatch} from "../../../../watchables/react/useWatch";
import {DefaultButton, Stack, useTheme} from "@fluentui/react";
import {CenteredContainer} from "../../../components/layout/CenteredContainer";
import {DiagramSummary} from "../DiagramSummary";
import {HttpDiagramCollectionState} from "../../../../state/diagrams/collections/HttpDiagramCollectionState";
import {DiagramCollectionContainer} from "./util/DiagramCollectionContainer";
import {useDragStart} from "../../../../utils/useDragStart";
import {useViewManager} from "../../../providers/ViewManagerContext";

export const HttpDiagramCollection: FC<{
    collection: HttpDiagramCollectionState;
    onDelete?: () => void;
}> = ({collection, onDelete}) => {
    const watch = useWatch();
    const theme = useTheme();

    const viewManager = useViewManager();
    const autoOpenTarget = collection.autoOpenTarget;
    const autoOpenRef = useDragStart((position, offset) => {
        const layout = viewManager.layoutState;
        const container = layout.allTabPanels
            .get()
            .find(c => c.tabs.some(({id}) => id == autoOpenTarget.ID));
        layout
            .setDraggingData({
                position,
                offset,
                removeFromPanelId: container?.id,
                preview: <AutoOpenButton />,
                targetId: autoOpenTarget.ID,
            })
            .commit();
    });
    const openAutoOpenTarget = useCallback(() => {
        viewManager.open(autoOpenTarget).commit();
    }, [autoOpenTarget]);

    return (
        <DiagramCollectionContainer
            title={collection.host}
            onDelete={onDelete}
            status={collection.status}>
            <Stack>
                {watch(collection.diagrams).map(diagram => (
                    <Stack.Item
                        align="stretch"
                        key={diagram.ID}
                        style={{marginBottom: theme.spacing.m}}>
                        <DiagramSummary
                            diagram={diagram}
                            onDelete={() => collection.removeDiagram(diagram).commit()}
                        />
                    </Stack.Item>
                ))}
            </Stack>
            <div ref={autoOpenRef}>
                <AutoOpenButton onClick={openAutoOpenTarget} />
            </div>
        </DiagramCollectionContainer>
    );
};

const AutoOpenButton: FC<{onClick?: () => void}> = ({onClick}) => {
    return (
        <DefaultButton
            onClick={onClick}
            style={{
                width: "100%",
            }}>
            Automatically open diagrams
        </DefaultButton>
    );
};
