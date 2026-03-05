import {DirectionHint, TooltipComp, TooltipDelay} from "oxidd-vis-rust";
import {NFC} from "../../../utils/_types/NFC";
import {StyledTooltipHost} from "../../components/StyledToolTipHost";
import React from "react";
import {ICompUI} from "../_types/ICompUI";
import {useWatch} from "../../../watchables/react/useWatch";
import {usePersistentMemo} from "../../../utils/usePersistentMemo";
import {v4 as uuid} from "uuid";
import {IAriaRef} from "../_types/IAriaRef";
import {addAriaDescription} from "../ariaRef";
import {DirectionalHint, TooltipDelay as FluentTooltipDelay} from "@fluentui/react";

export const TooltipCompUI: NFC<{
    data: TooltipComp;
    ChildComp: ICompUI;
    className?: string;
    aria?: IAriaRef;
}> = ({data, ChildComp, className, aria}) => {
    const id = usePersistentMemo(() => uuid(), []);
    const watch = useWatch();
    const tooltip = watch(data.tooltip);
    const content = watch(data.content);
    const delay = getDelay(watch(data.delay));
    const closeDelay = watch(data.close_delay);
    const direction = getDirection(watch(data.direction));
    return (
        <StyledTooltipHost
            id={id}
            delay={delay}
            closeDelay={closeDelay}
            directionalHint={direction}
            content={<ChildComp data={tooltip} />}>
            <ChildComp
                data={content}
                className={className}
                aria={addAriaDescription(id, aria)}
            />
        </StyledTooltipHost>
    );
};

function getDirection(dir: DirectionHint): DirectionalHint | undefined {
    switch (dir) {
        case DirectionHint.TopLeft:
            return DirectionalHint.topLeftEdge;
        case DirectionHint.TopCenter:
            return DirectionalHint.topCenter;
        case DirectionHint.TopRight:
            return DirectionalHint.topRightEdge;
        case DirectionHint.BottomLeft:
            return DirectionalHint.bottomLeftEdge;
        case DirectionHint.BottomCenter:
            return DirectionalHint.bottomCenter;
        case DirectionHint.BottomRight:
            return DirectionalHint.bottomRightEdge;
        case DirectionHint.LeftTop:
            return DirectionalHint.leftTopEdge;
        case DirectionHint.LeftCenter:
            return DirectionalHint.leftCenter;
        case DirectionHint.LeftBottom:
            return DirectionalHint.leftBottomEdge;
        case DirectionHint.RightTop:
            return DirectionalHint.rightTopEdge;
        case DirectionHint.RightCenter:
            return DirectionalHint.rightCenter;
        case DirectionHint.RightBottom:
            return DirectionalHint.rightBottomEdge;
        case DirectionHint.None:
            return undefined;
    }
}
function getDelay(delay: TooltipDelay): FluentTooltipDelay | undefined {
    switch (delay) {
        case TooltipDelay.Zero:
            return FluentTooltipDelay.zero;
        case TooltipDelay.Medium:
            return FluentTooltipDelay.medium;
        case TooltipDelay.Long:
            return FluentTooltipDelay.long;
    }
}
