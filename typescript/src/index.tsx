import ReactDOM from "react-dom";
import React, {FC, useCallback, useEffect, useRef} from "react";
import {DiagramDrawerBox, create_diagram} from "oxidd-viz-rust";
import {useTransformCallbacks} from "./useTransformCallbacks";
import {useNodeSelection} from "./useNodeSelection";

// Error.stackTraceLimit = 30;
Error.stackTraceLimit = 1;

const Test: FC = () => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const drawing = useRef<DiagramDrawerBox | null>(null);
    const start = useRef(Date.now());
    const computeTimeDelta = useRef(0);
    const gen = useCallback(() => {
        const canvas = canvasRef.current;
        if (canvas) {
            if (!drawing.current) {
                const diagram = create_diagram();
                const maybeDrawing = diagram?.create_drawer(canvas);
                if (maybeDrawing) drawing.current = maybeDrawing;
            }

            const d = drawing.current;
            const layoutStartTime = Date.now();
            d?.layout(layoutStartTime - start.current);
            computeTimeDelta.current = Date.now() - layoutStartTime;
            console.log("layed out");
        }
    }, []);
    useEffect(gen, []);

    const {
        handlers: {onMouseDown: tOnMouseDown, onWheel},
        transform,
    } = useTransformCallbacks(() => ({
        x: 0,
        y: 0,
        scale: 20,
    }));
    const {onMouseDown, onContextMenu, selection} = useNodeSelection(
        drawing,
        tOnMouseDown
    );
    const reveal = useCallback(() => {
        for (let group of selection) {
            drawing.current?.split_edges(group, false);
        }
        gen();
    }, [selection]);
    useEffect(() => {
        reveal();
    }, [selection]);

    useEffect(() => {
        const d = drawing.current;
        const c = canvasRef.current;
        if (d && c) {
            d.set_transform(c.width, c.height, transform.x, transform.y, transform.scale);
        }
    }, [transform]);

    useEffect(() => {
        let running = true;
        function render() {
            if (!running) return;
            const d = drawing.current;
            d?.render(
                Date.now() - start.current - computeTimeDelta.current,
                selection,
                new Uint32Array([])
            );
            requestAnimationFrame(render);
        }
        render();
        return () => void (running = false);
    }, [selection]);

    return (
        <div>
            <canvas
                height="900"
                width="700"
                ref={canvasRef}
                onWheel={onWheel}
                onContextMenu={onContextMenu}
                onMouseDown={onMouseDown}></canvas>
            <button onClick={gen}>regen</button>
            <button onClick={reveal}>reveal</button>
        </div>
    );
};

ReactDOM.render(<Test />, document.getElementById("root"));
