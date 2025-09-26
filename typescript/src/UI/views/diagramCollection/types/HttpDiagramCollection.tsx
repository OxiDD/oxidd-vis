import React, {FC, useCallback} from "react";
import {useWatch} from "../../../../watchables/react/useWatch";
import {
    DefaultButton,
    DirectionalHint,
    Label,
    MessageBar,
    MessageBarType,
    Stack,
    Toggle,
    useTheme,
} from "@fluentui/react";
import {CenteredContainer} from "../../../components/layout/CenteredContainer";
import {DiagramSummary} from "../DiagramSummary";
import {HttpDiagramCollectionState} from "../../../../state/diagrams/collections/HttpDiagramCollectionState";
import {DiagramCollectionContainer} from "./util/DiagramCollectionContainer";
import {useDragStart} from "../../../../utils/useDragStart";
import {useViewManager} from "../../../providers/ViewManagerContext";
import {StyledTooltipHost} from "../../../components/StyledToolTipHost";
import {usePersistentMemo} from "../../../../utils/usePersistentMemo";
import {v4 as uuid} from "uuid";

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

    const replaceDiagramTooltipID = usePersistentMemo(() => uuid(), []);
    const openDiagramTooltipID = usePersistentMemo(() => uuid(), []);
    const diagrams = watch(collection.diagrams);
    return (
        <DiagramCollectionContainer
            title={collection.host}
            onDelete={onDelete}
            status={collection.status}>
            <Stack>
                {diagrams.map(diagram => (
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
            {diagrams.length == 0 && watch(collection.status) == undefined && (
                <div style={{marginBottom: theme.spacing.m}}>
                    Host has no diagrams, or is unreachable from this browser
                </div>
            )}
            <Stack horizontal tokens={{childrenGap: theme.spacing.m}}>
                <div ref={autoOpenRef} style={{flexGrow: 1}}>
                    <StyledTooltipHost
                        directionalHint={DirectionalHint.topCenter}
                        id={openDiagramTooltipID}
                        content="Select a panel to automatically open the diagrams into">
                        <AutoOpenButton
                            aria={openDiagramTooltipID}
                            onClick={openAutoOpenTarget}
                        />
                    </StyledTooltipHost>
                </div>
                <div>
                    <StyledTooltipHost
                        directionalHint={DirectionalHint.topCenter}
                        id={replaceDiagramTooltipID}
                        content="Replace old diagrams when a new diagram with the same name loads">
                        <Stack
                            horizontal
                            tokens={{childrenGap: theme.spacing.s1}}
                            verticalAlign="center"
                            style={{height: "100%"}}>
                            <Label>Replace diagrams</Label>
                            <Toggle
                                checked={watch(collection.replaceOld)}
                                aria-describedby={replaceDiagramTooltipID}
                                styles={{root: {marginBottom: 0}}}
                                onChange={(_, c) =>
                                    collection.replaceOld.set(c!).commit()
                                }
                            />
                        </Stack>
                    </StyledTooltipHost>
                </div>
            </Stack>
        </DiagramCollectionContainer>
    );
};

const AutoOpenButton: FC<{onClick?: () => void; aria?: string}> = ({onClick, aria}) => {
    return (
        <DefaultButton
            onClick={onClick}
            aria-describedby={aria}
            style={{
                width: "100%",
            }}>
            Open diagram panel
        </DefaultButton>
    );
};
