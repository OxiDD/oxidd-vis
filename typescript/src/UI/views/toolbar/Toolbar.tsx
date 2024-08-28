import React, {FC} from "react";
import {ToolbarState} from "../../../state/toolbar/ToolbarState";
import {Pivot, PivotItem} from "@fluentui/react";
import {useWatch} from "../../../watchables/react/useWatch";
import {IToolName} from "../../../state/toolbar/_types/IToolName";

export const Toolbar: FC<{toolbar: ToolbarState}> = ({toolbar}) => {
    const watch = useWatch();
    return (
        <Pivot
            aria-label="Toolbar"
            selectedKey={watch(toolbar.selectedToolName)}
            onLinkClick={e =>
                toolbar.selectedToolName.set(e?.props.itemKey as IToolName).commit()
            }>
            <PivotItem itemIcon="ClearSelection" itemKey="selection" />
            <PivotItem itemIcon="MaximumValue" itemKey="expansion" />
        </Pivot>
    );
};
