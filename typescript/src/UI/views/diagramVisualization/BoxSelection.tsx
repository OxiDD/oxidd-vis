import React, {FC, useCallback, useRef, useState} from "react";
import {css} from "@emotion/css";
import {IRectangle} from "../../../utils/_types/IRectangle";
import {useTheme} from "@fluentui/react";

/**  A component that handles selecting a box within its parent  */
export const BoxSelection: FC<{
    onSelect: (rect: IRectangle) => void;
    onHighlight?: (rect: IRectangle) => void;
    onStart?: (event: React.MouseEvent<HTMLDivElement, MouseEvent>) => boolean;
}> = ({onSelect, onHighlight, onStart}) => {
    const theme = useTheme();
    const containerRef = useRef<HTMLDivElement>(null);
    const [selection, setSelection] = useState<IRectangle | null>(null);
    const selectionRef = useRef<IRectangle | null>(null);
    const onDown = useCallback((event: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
        const select = onStart?.(event) ?? true;
        if (!select) return;
        const frame = containerRef.current?.getBoundingClientRect();
        if (!frame) return;
        event.preventDefault();
        event.stopPropagation();

        const start = {
            x: event.clientX - frame.left,
            y: event.clientY - frame.top,
        };
        const selection = {
            left_top: start,
            size: {x: 0, y: 0},
        };
        selectionRef.current = selection;
        setSelection(selection);
        onHighlight?.(selection);

        const moveListener = (e: MouseEvent) => {
            const x = e.clientX - frame.left;
            const y = e.clientY - frame.top;
            const minX = Math.min(start.x, x);
            const maxX = Math.max(start.x, x);
            const minY = Math.min(start.y, y);
            const maxY = Math.max(start.y, y);
            const selection = {
                left_top: {x: minX, y: minY},
                size: {x: maxX - minX, y: maxY - minY},
            };
            selectionRef.current = selection;
            setSelection(selection);
            onHighlight?.(selection);
        };
        const upListener = (e: MouseEvent) => {
            const selection = selectionRef.current;
            if (selection) onSelect(selection);
            setSelection(null);
            window.removeEventListener("mousemove", moveListener);
            window.removeEventListener("mouseup", upListener);
        };
        window.addEventListener("mousemove", moveListener);
        window.addEventListener("mouseup", upListener);
    }, []);

    return (
        <div
            ref={containerRef}
            onMouseDown={onDown}
            className={css({
                position: "absolute",
                overflow: "hidden",
                left: 0,
                right: 0,
                top: 0,
                bottom: 0,
            })}>
            {selection && (
                <div
                    className={css({
                        position: "absolute",
                        overflow: "hidden",
                        border: "1px solid",
                        borderRadius: 3,
                        borderColor: theme.palette.themePrimary,
                    })}
                    style={{
                        left: selection.left_top.x,
                        top: selection.left_top.y,
                        width: selection.size.x,
                        height: selection.size.y,
                    }}>
                    <div
                        className={css({
                            backgroundColor: theme.palette.themePrimary,
                            opacity: 0.4,
                            position: "absolute",
                            left: 0,
                            right: 0,
                            top: 0,
                            bottom: 0,
                        })}
                    />
                </div>
            )}
            hello
        </div>
    );
};
