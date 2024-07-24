import React, {FC, forwardRef, useEffect, useMemo, useRef} from "react";
import {mergeRefs} from "react-merge-refs";
import {ITabsContentProps} from "../_types/props/ITabsContentProps";
import {css} from "@emotion/css";

export const TabsContent: FC<ITabsContentProps> = ({contents}) => (
    <>
        {contents.map(({id, element, selected}) => (
            <TabTarget key={id} selected={selected} element={element} />
        ))}
    </>
);

export const TabTarget = forwardRef<
    HTMLDivElement,
    {selected: boolean; element: HTMLElement} & React.HTMLAttributes<HTMLDivElement>
>(({selected, element, ...rest}, fRef) => {
    const lRef = useRef<HTMLDivElement>(null);
    useEffect(() => {
        const el = lRef.current;
        if (!el) return;

        el.appendChild(element);
    }, [lRef]);

    return (
        <div
            ref={mergeRefs([lRef, fRef])}
            className={css({
                flexGrow: 1,
                flexShrink: 1,
                minHeight: 0,
                position: "relative",
                display: selected ? "flex" : "none",
                justifyItems: "stretch",
                "&>div": {
                    flexGrow: 1,
                    width: "100%",
                    minWidth: 0,
                },
            })}
            {...rest}
        />
    );
});
