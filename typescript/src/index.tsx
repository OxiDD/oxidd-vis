import ReactDOM from "react-dom";
import React, {FC, useCallback, useEffect, useRef} from "react";
import {DiagramDrawerBox, create_diagram} from "oxidd-viz-rust";

const Test: FC = () => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const drawing = useRef<DiagramDrawerBox | null>(null);
    const start = useRef(Date.now());
    const gen = useCallback(() => {
        Error.stackTraceLimit = 30;
        const canvas = canvasRef.current;
        if (canvas) {
            if (!drawing.current) {
                const diagram = create_diagram();
                const maybeDrawing = diagram?.create_drawer(canvas);
                if (maybeDrawing) drawing.current = maybeDrawing;
            }

            const d = drawing.current;
            d?.layout(Date.now() - start.current);
            d?.set_transform(0, 0, 0.1);

            console.log("Success??", d);
        }
    }, []);
    useEffect(gen, []);
    useEffect(() => {
        const id = setInterval(() => {
            const d = drawing.current;
            d?.render(
                Date.now() - start.current,
                new Uint32Array([]),
                new Uint32Array([])
            );
        }, 0);
        return () => clearInterval(id);
    }, []);

    return (
        <div onClick={gen}>
            <canvas height="600" width="600" ref={canvasRef}></canvas>
            Hello world
        </div>
    );
};

ReactDOM.render(<Test />, document.getElementById("root"));
