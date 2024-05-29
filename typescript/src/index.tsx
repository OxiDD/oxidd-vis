import ReactDOM from "react-dom";
import React, {FC, useCallback, useEffect, useRef} from "react";
import {DiagramDrawerBox, create_diagram} from "oxidd-viz-rust";
import {useTransformCallbacks} from "./useTransformCallbacks";

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
    useEffect(() => {
        let running = true;
        function render() {
            if (!running) return;
            const d = drawing.current;
            d?.render(
                Date.now() - start.current - computeTimeDelta.current,
                new Uint32Array([]),
                new Uint32Array([])
            );
            requestAnimationFrame(render);
        }
        render();
        return () => void (running = false);
    }, []);

    const {handlers, transform} = useTransformCallbacks(() => ({
        x: 0,
        y: 0,
        scale: 0.02,
    }));
    useEffect(() => {
        const d = drawing.current;
        if (d) {
            d.set_transform(transform.x, transform.y, transform.scale);
        }
    }, [transform]);

    return (
        <div>
            <canvas height="900" width="1400" ref={canvasRef} {...handlers}></canvas>
            <button onClick={gen}>regen</button>
        </div>
    );
};

ReactDOM.render(<Test />, document.getElementById("root"));
