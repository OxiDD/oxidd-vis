import React from "react";
import {NFC} from "../../../utils/_types/NFC";
import {PanelHandleComp} from "oxidd-vis-rust";
import {ICompUI} from "../_types/ICompUI";
import {useWatch} from "../../../watchables/react/useWatch";
import {useDragStart} from "../../../utils/useDragStart";
import {useViewManager} from "../../providers/ViewManagerContext";
import {IAriaRef} from "../_types/IAriaRef";

export const PanelHandleCompUI: NFC<{
    data: PanelHandleComp;
    ChildComp: ICompUI;
    className?: string;
    aria?: IAriaRef;
}> = ({data, ChildComp, className, aria}) => {
    const watch = useWatch();
    const main = watch(data.main);

    const content = <ChildComp data={main} aria={aria} />;
    const viewID = data.panel.id;
    const viewManager = useViewManager();
    const ref = useDragStart((position, offset) => {
        const layout = viewManager.layoutState;
        const container = layout.allTabPanels
            .get()
            .find(c => c.tabs.some(({id}) => id == viewID));
        layout
            .setDraggingData({
                position,
                offset,
                targetId: viewID,
                removeFromPanelId: container?.id,
                preview: content,
            })
            .commit();
    });
    return (
        <div className={className} ref={ref}>
            {content}
        </div>
    );
};
