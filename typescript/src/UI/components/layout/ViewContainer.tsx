import React, {FC, ReactNode, forwardRef} from "react";
import {CSSInterpolation, css} from "@emotion/css";
import {useScrollbarStyle} from "../../hooks/useScrollbarStyle";

/** Standard styling for the panel contents container */
export const ViewContainer = forwardRef<
    HTMLDivElement,
    React.HTMLAttributes<HTMLDivElement> & {css?: CSSInterpolation}
>(({className, children, css: cssArg, ...rest}, ref) => {
    const scrollbarStyle = useScrollbarStyle();
    return (
        <div
            ref={ref}
            {...rest}
            className={`${css(
                {
                    padding: 10,
                    boxSizing: "border-box",
                    height: "100%",
                    width: "100%",
                    overflow: "auto",
                    ...scrollbarStyle,
                },
                cssArg
            )} ${className}`}>
            {children}
        </div>
    );
});
