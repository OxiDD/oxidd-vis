import ReactDOM from "react-dom";
import React, {FC, useRef} from "react";
import {create_diagram} from "oxidd-viz-rust";

const Test: FC = () => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    return (
        <div
            onClick={() => {
                Error.stackTraceLimit = 30;
                const canvas = canvasRef.current;
                if (canvas) {
                    const diagram = create_diagram();
                    const drawing = diagram?.create_drawer(canvas);
                    drawing?.render(Date.now(), new Uint32Array([]), new Uint32Array([]));
                    console.log("Success??", drawing);
                }
            }}>
            <canvas height="300" width="300" ref={canvasRef}></canvas>
            Hello world
        </div>
    );
};

ReactDOM.render(<Test />, document.getElementById("root"));
