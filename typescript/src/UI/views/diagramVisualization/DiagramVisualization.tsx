import React, {FC, useCallback, useEffect, useLayoutEffect, useRef} from "react";
import {DiagramVisualizationState} from "../../../state/diagrams/DiagramVisualizationState";
import {useTransformCallbacks} from "./useTransformCallbacks";
import {css} from "@emotion/css";
import {ViewContainer} from "../../components/layout/ViewContainer";
import {BoxSelection} from "./BoxSelection";
import {useToolbar} from "../../providers/ToolbarContext";
import {useWatch} from "../../../watchables/react/useWatch";
import {ActionButton, PrimaryButton, useTheme} from "@fluentui/react";
import {Toolbar} from "../toolbar/Toolbar";
import {PresenceRemainder} from "oxidd-viz-rust";
import {ConfigTypeComp} from "../../components/configuration/ConfigTypeComp";

export const DiagramVisualization: FC<{visualization: DiagramVisualizationState}> = ({
    visualization,
}) => {
    const theme = useTheme();
    const watch = useWatch();
    const toolbar = useToolbar();
    const ref = useRef<HTMLDivElement>(null);
    useLayoutEffect(() => {
        const el = ref.current;
        if (el) {
            const setSize = () => {
                const width = el.clientWidth;
                const height = el.clientHeight;
                if (width <= 0 || height <= 0) return;
                visualization.size.set({x: width, y: height}).commit();
            };
            setSize();
            const resizeObserver = new ResizeObserver(() => setTimeout(setSize)); // timeout used to prevent UI updates resulting from UI size change
            resizeObserver.observe(el);
            return () => resizeObserver.disconnect();
        }
    }, []);
    useEffect(() => {
        const el = ref.current;
        if (el) {
            el.insertBefore(visualization.canvas, el.firstChild);

            let running = true;
            function render() {
                if (!running) return;
                visualization.render();
                requestAnimationFrame(render);
            }
            render();
            return () => {
                running = false;
            };
        }
    }, []);

    // Prevent dragging the window when clicking a button
    const preventDrag = useCallback((e: React.MouseEvent) => {
        e.stopPropagation();
    }, []);
    const moveListeners = useTransformCallbacks(visualization.transform);
    const modeRef = useRef(0);
    return (
        <ViewContainer
            onContextMenu={e => e.preventDefault()}
            ref={ref}
            {...moveListeners}
            css={{padding: 0, overflow: "hidden", backgroundColor: "white"}}>
            <BoxSelection
                onStart={m => m.buttons == 1}
                onHighlight={(rect, e) => {
                    const nodes = visualization.getNodes(rect);
                    visualization.applyTool(toolbar, nodes, {
                        type: "drag",
                        event: e,
                    });
                }}
                onSelect={(rect, e) => {
                    const nodes = visualization.getNodes(rect);
                    visualization.applyTool(toolbar, nodes, {
                        type: "release",
                        event: e,
                    });
                }}></BoxSelection>
            <div
                onMouseDown={preventDrag}
                className={css({
                    position: "absolute",
                    right: theme.spacing.m,
                    top: theme.spacing.m,
                    background: theme.palette.neutralLight,
                })}>
                <Toolbar toolbar={toolbar} visualization={visualization} />
            </div>
            <div
                onMouseDown={preventDrag}
                className={css({
                    position: "absolute",
                    right: theme.spacing.m,
                    bottom: theme.spacing.m,
                    background: theme.palette.neutralLight,
                })}>
                <ConfigTypeComp value={watch(visualization.config)} />
            </div>
        </ViewContainer>
    );
};
