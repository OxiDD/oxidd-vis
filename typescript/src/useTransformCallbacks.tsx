import {useCallback, useRef, useState} from "react";
import {transition} from "./transition";

export function useTransformCallbacks(init: () => ITransform): {
    handlers: IInteractionHandlers;
    transform: ITransform;
} {
    const [transform, setTransform] = useState(init);
    const scaleTarget = useRef(transform.scale);
    const transformRef = useRef(transform);
    const stopPrevScaleTransition = useRef(() => {});
    transformRef.current = transform;
    const getTargetPoint = (e: React.WheelEvent<HTMLCanvasElement>) => {
        const canvas = e.target as HTMLCanvasElement;
        const bound = canvas.getBoundingClientRect();
        const elX = e.pageX - bound.x;
        const elY = e.pageY - bound.y;
        return {
            x: elX / bound.width - 0.5,
            y: -(elY / bound.height - 0.5),
        };
    };
    const setScale = (newScale: number, target: IPoint) => {
        setTransform(({x, y, scale}) => ({
            x: x + (target.x / newScale - target.x / scale),
            y: y + (target.y / newScale - target.y / scale),
            scale: newScale,
        }));
    };

    return {
        transform,
        handlers: {
            onWheel: useCallback(e => {
                const delta = e.deltaX + e.deltaY + e.deltaZ;
                const multiplier = 1.3 ** (-delta / 100);
                const target = getTargetPoint(e);
                const startScale = transformRef.current.scale;
                const endScale = (scaleTarget.current *= multiplier);

                stopPrevScaleTransition.current();
                stopPrevScaleTransition.current = transition(per => {
                    setScale((1 - per) * startScale + per * endScale, target);
                }, 100).cancel;
            }, []),
            onMouseDown: useCallback(e => {
                const target = e.target as HTMLCanvasElement;
                // We register listeners on the window, such that dragging works even when leaving the canvas
                const moveListener = (e: MouseEvent) => {
                    if ((e.buttons & 1) != 0) {
                        const width = target.width;
                        const height = target.height;
                        setTransform(({x, y, scale}) => ({
                            x: x + e.movementX / width / scale,
                            y: y - e.movementY / height / scale,
                            scale,
                        }));
                    }
                };
                const upListener = (e: MouseEvent) => {
                    window.removeEventListener("mousemove", moveListener);
                    window.removeEventListener("mouseup", upListener);
                };
                window.addEventListener("mousemove", moveListener);
                window.addEventListener("mouseup", upListener);
            }, []),
        },
    };
}

type IPoint = {x: number; y: number};
type ITransform = IPoint & {scale: number};
type IInteractionHandlers = {
    onWheel: React.WheelEventHandler<HTMLCanvasElement>;
    onMouseDown: (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => void;
    // onMouseUp: (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => void;
    // onMouseMove: (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => void;
};
