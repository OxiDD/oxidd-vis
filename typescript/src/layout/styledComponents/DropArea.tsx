import React, {FC} from "react";
import {IDropAreaProps} from "../_types/props/IDropAreaProps";
import {getTheme, mergeStyles, useTheme} from "@fluentui/react";
import {IDropPanelSide} from "../_types/IDropSide";
import {css} from "@emotion/css";

export const DropArea: FC<IDropAreaProps> = ({dragging, onDrop}) => {
    const theme = useTheme();
    const dropAreaColor = theme.palette.themeTertiary;
    const dropColor = theme.palette.themePrimary;
    return (
        <>
            {dragging && (
                <div
                    className={css({
                        "& .drop": {backgroundColor: dropAreaColor},
                        position: "absolute",
                        left: 0,
                        top: 42,
                        right: 0,
                        bottom: 0,
                        display: "flex",
                        justifyContent: "center",
                        alignItems: "center",
                        opacity: 0,
                        "&:hover": {
                            opacity: 1,
                        },

                        "& .drop:hover": {backgroundColor: dropColor},
                        ".background": {
                            // opacity: 0.7,
                            opacity: 0.0,
                            backgroundColor: theme.palette.white,
                        },
                    })}>
                    <Diagonals
                        size={200}
                        innerSize={75}
                        spacing={10}
                        diagonalSpacing={10}
                        onDrop={onDrop}
                    />
                </div>
            )}
        </>
    );
};

const DiagonalCorner: FC<{angle: number; inset: number; onDrop: () => void}> = ({
    angle,
    inset,
    onDrop,
}) => (
    <div
        style={{
            flexGrow: 1,
            display: "flex",
            justifyContent: "center",
            alignItems: "center",
            transform: `rotate(${angle}deg)`,
            overflow: "hidden",
        }}>
        <div
            className="drop"
            onMouseUp={onDrop}
            style={{
                transform: `scale(${Math.sqrt(2) * 100}%) rotate(45deg) translate(50%)`,
                // width: `${((0.5 * 1) / Math.sqrt(2)) * 100}%`,
                width: `${(1 - inset) * 50}%`,
                height: "100%",
            }}
        />
    </div>
);

export const Diagonals: FC<{
    size: number;
    innerSize: number;
    spacing: number;
    diagonalSpacing: number;
    onDrop: (side: IDropPanelSide) => void;
}> = ({size, innerSize, spacing, diagonalSpacing, onDrop}) => {
    const sqrt2 = Math.sqrt(2);

    // const diagonalSpacing = sqrt2 * spacing;
    const diagonalSpacingOr = diagonalSpacing / sqrt2;
    const diagonalsSize = sqrt2 * size - diagonalSpacing;
    const inset =
        ((2 * innerSize) / sqrt2 - diagonalSpacing + 2 * spacing) / diagonalsSize;

    return (
        <div style={{position: "relative"}}>
            <div
                className="background"
                style={{
                    width: size,
                    height: size,
                    position: "absolute",
                    top: "50%",
                    left: "50%",
                    transform: "translate(-50%, -50%)",
                }}></div>
            <div
                style={{
                    width: diagonalsSize,
                    height: diagonalsSize,
                    display: "flex",
                    flexDirection: "column",
                    gap: diagonalSpacingOr,
                    transform: "rotate(45deg)",
                }}>
                <div
                    style={{
                        display: "flex",
                        flexGrow: 1,
                        flexDirection: "row",
                        gap: diagonalSpacingOr,
                    }}>
                    <DiagonalCorner
                        angle={0}
                        inset={inset}
                        onDrop={() => onDrop("north")}
                    />
                    <DiagonalCorner
                        angle={90}
                        inset={inset}
                        onDrop={() => onDrop("east")}
                    />
                </div>
                <div
                    style={{
                        display: "flex",
                        flexGrow: 1,
                        flexDirection: "row",
                        gap: diagonalSpacingOr,
                    }}>
                    <DiagonalCorner
                        angle={270}
                        inset={inset}
                        onDrop={() => onDrop("west")}
                    />
                    <DiagonalCorner
                        angle={180}
                        inset={inset}
                        onDrop={() => onDrop("south")}
                    />
                </div>
            </div>
            <div
                className="drop"
                onMouseUp={() => onDrop("in")}
                style={{
                    width: innerSize,
                    height: innerSize,
                    position: "absolute",
                    top: "50%",
                    left: "50%",
                    transform: "translate(-50%, -50%)",
                }}
            />
        </div>
    );
};
