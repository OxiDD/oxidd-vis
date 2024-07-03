import React, {FC, useEffect, CSSProperties} from "react";
import {IDragPreviewProps} from "../_types/props/IDragPreviewProps";
import {Overlay} from "@fluentui/react";

export const DragPreview: FC<
    IDragPreviewProps & {DragWrapper?: typeof DefaultDragWrapper}
> = ({data, state, DragWrapper = DefaultDragWrapper}) => {
    useEffect(() => {
        if (!data) return;

        const moveListener = (event: MouseEvent) => {
            const latestData = state.draggingData.get();
            if (!latestData) return;
            state
                .setDraggingData({
                    ...latestData,
                    position: {x: event.pageX, y: event.pageY},
                })
                .commit();
        };
        const resetListener = () => {
            state.setDraggingData(null).commit();
        };
        window.addEventListener("mousemove", moveListener);
        window.addEventListener("mouseup", resetListener);
        return () => {
            window.removeEventListener("mousemove", moveListener);
            window.removeEventListener("mouseup", resetListener);
        };
    }, [!!data]);

    if (!data) return <></>;
    return (
        <Overlay styles={{root: {background: "none", pointerEvents: "none"}}}>
            <DragWrapper
                style={{
                    position: "absolute",
                    top: data.position.y + data.offset.y,
                    left: data.position.x + data.offset.x,
                }}>
                {data.preview}
            </DragWrapper>
        </Overlay>
    );
};

export const DefaultDragWrapper: FC<{style: CSSProperties}> = ({style, children}) => (
    <div style={{...style, opacity: 0.8, zIndex: 200}}>{children}</div>
);
