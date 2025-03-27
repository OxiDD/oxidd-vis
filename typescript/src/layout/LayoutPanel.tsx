import React, {FC, useRef, useMemo, useEffect} from "react";
import {LayoutState} from "./LayoutState";
import {IPanelData, IPanelSplitData, IPanelTabsData} from "./_types/IPanelData";
import {
    PanelGroup,
    PanelResizeHandle,
    Panel,
    ImperativePanelGroupHandle,
} from "react-resizable-panels";
import {intersperse, intersperseDynamic} from "../utils/intersperse";
import {IPanelSplitState, IPanelState, IPanelTabsState} from "./_types/IPanelState";
import {ILayoutComponents} from "./_types/ILayourComponents";
import {IContent, IContentGetter} from "./_types/IContentGetter";
import {ITabData} from "./_types/props/ITabsHeaderProps";
import {DropArea} from "./styledComponents/DropArea";
import {IDropPanelSide} from "./_types/IDropSide";
import {v4 as uuid} from "uuid";
import {usePersistentMemo} from "../utils/usePersistentMemo";
import {useChangeID} from "../utils/useChangeID";
import {usePrevious} from "../utils/usePrevious";
import {useWatch} from "../watchables/react/useWatch";
import {chain} from "../watchables/mutator/chain";
import {css} from "@emotion/css";

/**
 * The component for rendering a layout panel
 */
export const LayoutPanel: FC<{
    state: LayoutState;
    panel: IPanelState;
    getContent: IContentGetter;
    components: ILayoutComponents;
    isRoot?: boolean;
}> = props => {
    if (props.panel.type == "split")
        return <LayoutSplitPanel {...props} panel={props.panel} />;
    return <LayoutTabsPanel {...props} panel={props.panel} />;
};

export const LayoutSplitPanel: FC<{
    state: LayoutState;
    panel: IPanelSplitState;
    getContent: IContentGetter;
    components: ILayoutComponents;
}> = ({state, panel, components, getContent}) => {
    // We need to force a new key sometimes, since panel groups apparently can bug out when the number of panels changes, so a new key fully resets them
    const childIds = panel.panels.map(p => p.content.id);
    const prevChildIds = usePrevious(childIds);
    const key = useChangeID(
        childIds.length != prevChildIds.length ||
            childIds.some((value, i) => value != prevChildIds[i])
    );

    return (
        <PanelGroup direction={panel.direction} ref={panel.handle} key={key}>
            {intersperseDynamic(
                panel.panels.map(panel => (
                    <Panel
                        key={panel.content.id}
                        minSize={3}
                        // className={css({minHeight: 30})}
                        defaultSize={panel.defaultWeight}>
                        <LayoutPanel
                            state={state}
                            panel={panel.content}
                            components={components}
                            getContent={getContent}
                        />
                    </Panel>
                )),
                i => (
                    <PanelResizeHandle key={panel.panels[i].content.id + "-sep"}>
                        <components.ResizeHandle
                            direction={panel.direction}
                            state={state}
                        />
                    </PanelResizeHandle>
                )
            )}
        </PanelGroup>
    );
};

export const LayoutTabsPanel: FC<{
    state: LayoutState;
    panel: IPanelTabsState;
    getContent: IContentGetter;
    components: ILayoutComponents;
    isRoot?: boolean;
}> = ({state, panel, getContent, components, isRoot: root}) => {
    const watch = useWatch();
    const isDragging = !!watch(state.draggingData);
    const orderedContents = panel.tabs
        .map(({id}) => watch(getContent(id)))
        .filter((v): v is IContent => v != undefined);

    const tabData = orderedContents.flatMap<ITabData>(
        ({id, content, forceOpen, ...rest}) => {
            const elData = panel.tabs.find(tab => tab.id == id);
            if (!elData) return [];
            return [
                {
                    id,
                    element: elData.element,
                    ...rest,
                    selected: id == panel.selected,
                    forceOpen: forceOpen ?? false,
                },
            ];
        }
    );

    const onDropTab = (beforeId: string) => {
        chain(push => {
            const dragging = state.draggingData.get();
            if (!dragging?.targetId) return;
            if (
                dragging.removeFromPanelId == panel.id &&
                (dragging.targetId == beforeId || panel.tabs.length == 1)
            )
                return; // Nothing should change
            if (dragging.removeFromPanelId)
                push(state.closeTab(dragging.removeFromPanelId, dragging.targetId));
            push(
                state.openTab(
                    panel.id,
                    dragging.target ?? dragging.targetId,
                    undefined,
                    beforeId
                )
            );
            push(state.selectTab(panel.id, dragging.targetId));
        }).commit();
    };

    const onDropSide = (side: IDropPanelSide) => {
        chain(push => {
            const dragging = state.draggingData.get();
            if (!dragging?.targetId) return;

            // Insert in this tab
            if (side == "in") {
                if (dragging.removeFromPanelId == panel.id) return; // Nothing should change
                if (dragging.removeFromPanelId)
                    push(state.closeTab(dragging.removeFromPanelId, dragging.targetId));
                push(state.openTab(panel.id, dragging.target ?? dragging.targetId));
                push(state.selectTab(panel.id, dragging.targetId));
                return;
            }

            // Or add a new panel
            const targetPanelId = push(state.addPanel(panel.id, side));
            if (targetPanelId) {
                if (dragging.removeFromPanelId)
                    push(state.closeTab(dragging.removeFromPanelId, dragging.targetId));
                push(state.openTab(targetPanelId, dragging.target ?? dragging.targetId));
            }
        }).commit();
    };

    return (
        <components.TabsContainer state={state}>
            <components.TabsHeader
                tabs={tabData}
                onClose={
                    root
                        ? undefined
                        : () => {
                              chain(push => {
                                  for (const {id} of panel.tabs)
                                      push(state.closeTab(panel.id, id));
                                  push(state.removePanel(panel.id));
                              }).commit();
                          }
                }
                onSelectTab={id => state.selectTab(panel.id, id).commit()}
                onCloseTab={id => state.closeTab(panel.id, id).commit()}
                onDragStart={data =>
                    state
                        .setDraggingData({
                            offset: {x: 0, y: 0},
                            removeFromPanelId: panel.id,
                            ...data,
                        })
                        .commit()
                }
                selectedTab={panel.selected}
                dragging={isDragging}
                onDrop={onDropTab}
                state={state}
            />
            <components.TabsContent
                contents={tabData.map(({id, ...rest}) => ({
                    id,
                    selected: panel.selected == id,
                    ...rest,
                }))}
                state={state}
            />
            <components.DropArea
                dragging={isDragging}
                onDrop={onDropSide}
                state={state}
            />
        </components.TabsContainer>
    );
};
