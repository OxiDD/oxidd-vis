import {DiagramDrawerBox} from "oxidd-viz-rust";
import {useCallback, useState} from "react";

export function useNodeSelection(
    drawer: React.MutableRefObject<DiagramDrawerBox | null>,
    onMouseDown?: (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => void
): {
    onContextMenu: React.MouseEventHandler<HTMLCanvasElement> | undefined;
    onMouseDown: (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => void;
    selection: Uint32Array;
} {
    const [selection, setSelection] = useState(() => new Uint32Array());
    return {
        onMouseDown: useCallback(
            e => {
                const d = drawer.current;
                if ((e.buttons & 2) != 0 && d) {
                    const canvas = e.target as HTMLCanvasElement;
                    const bound = canvas.getBoundingClientRect();
                    const elX = e.pageX - bound.x;
                    const elY = e.pageY - bound.y;
                    setSelection(
                        d.get_nodes(
                            elX / bound.width - 0.5,
                            -(elY / bound.height - 0.5),
                            0,
                            0
                        )
                    );
                    e.preventDefault();
                } else {
                    onMouseDown?.(e);
                }
            },
            [drawer.current]
        ),
        onContextMenu: useCallback(
            e => {
                e.preventDefault();
            },
            [drawer.current]
        ),
        selection,
    };
}
