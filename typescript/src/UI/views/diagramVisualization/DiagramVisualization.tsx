import React, {FC, useEffect, useRef} from "react";
import {DiagramVisualizationState} from "../../../state/diagrams/DiagramVisualizationState";
import {useTransformCallbacks} from "./useTransformCallbacks";
import {css} from "@emotion/css";
import {ViewContainer} from "../../components/ViewContainer";

export const DiagramVisualization: FC<{visualization: DiagramVisualizationState}> = ({
    visualization,
}) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        const el = ref.current;
        if (el) {
            const observer = new ResizeObserver(() => {
                visualization.size.set({x: el.clientWidth, y: el.clientHeight}).commit();
            });
            observer.observe(el);
            return () => observer.disconnect();
        }
    }, []);
    useEffect(() => {
        const el = ref.current;
        if (el) {
            el.appendChild(visualization.canvas);

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
    return <ViewContainer ref={ref} {...moveListeners} className={css({padding: 0})} />;
};
