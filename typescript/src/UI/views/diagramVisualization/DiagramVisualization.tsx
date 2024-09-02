import React, {FC, useEffect, useRef} from "react";
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

export const DiagramVisualization: FC<{visualization: DiagramVisualizationState}> = ({
    visualization,
}) => {
    const theme = useTheme();
    const watch = useWatch();
    const toolbar = useToolbar();
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
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
    const moveListeners = useTransformCallbacks(visualization.transform);
    const modeRef = useRef(0);
    return (
        <ViewContainer
            onContextMenu={e => e.preventDefault()}
            ref={ref}
            {...moveListeners}
            css={{padding: 0, overflow: "hidden"}}>
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
                className={css({
                    position: "absolute",
                    right: theme.spacing.m,
                    top: theme.spacing.m,
                    background: theme.palette.neutralLight,
                })}>
                <Toolbar toolbar={toolbar} />

                <PrimaryButton
                    text="toggle"
                    onClick={() => {
                        visualization.applyTool(
                            {
                                apply(visualization, drawer, nodes, event) {
                                    let m = (modeRef.current = (modeRef.current + 1) % 3);
                                    drawer.set_terminal_mode(
                                        "F",
                                        m == 0
                                            ? PresenceRemainder.Show
                                            : m == 1
                                            ? PresenceRemainder.Duplicate
                                            : PresenceRemainder.Hide
                                    );
                                    return true;
                                },
                            },
                            visualization.sharedState.selection.get()
                        );
                    }}
                />
            </div>
        </ViewContainer>
    );
};
