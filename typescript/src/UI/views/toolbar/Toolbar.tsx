import React, {FC} from "react";
import {ToolbarState} from "../../../state/toolbar/ToolbarState";
import {DirectionalHint, Icon, IPivotItemProps, Pivot, PivotItem} from "@fluentui/react";
import {useWatch} from "../../../watchables/react/useWatch";
import {IToolName} from "../../../state/toolbar/_types/IToolName";
import {DiagramVisualizationState} from "../../../state/diagrams/DiagramVisualizationState";
import {StyledTooltipHost} from "../../components/StyledToolTipHost";
import {css} from "@emotion/css";

export const Toolbar: FC<{
    toolbar: ToolbarState;
    visualization?: DiagramVisualizationState;
}> = ({toolbar, visualization}) => {
    const watch = useWatch();
    return (
        <Pivot
            aria-label="Toolbar"
            className={css({"[role='tab']": {padding: 0, marginRight: 0}})}
            selectedKey={watch(toolbar.selectedToolName)}
            onLinkClick={e =>
                toolbar.selectedToolName.set(e?.props.itemKey as IToolName).commit()
            }>
            <PivotItem
                itemIcon="ClearSelection"
                itemKey="selection"
                title="Select nodes"
                onRenderItemLink={TooltipPivot}
            />
            <PivotItem
                itemIcon="Split"
                itemKey="expansion"
                title="Expand children of nodes"
                onRenderItemLink={TooltipPivot}
            />
            <PivotItem
                itemIcon="Combine"
                itemKey="grouping"
                title="Combine nodes"
                onRenderItemLink={TooltipPivot}
            />
        </Pivot>
    );
};

// export const ItemRenderer =

const TooltipPivot = (
    link?: IPivotItemProps,
    defaultRenderer?: (link?: IPivotItemProps) => JSX.Element | null
): JSX.Element | null => {
    if (!link || !defaultRenderer) {
        return null;
    }

    return (
        <StyledTooltipHost
            directionalHint={DirectionalHint.leftCenter}
            content={link.title}>
            <span style={{marginLeft: 12, marginRight: 12}}>{defaultRenderer(link)}</span>
        </StyledTooltipHost>
    );
};
