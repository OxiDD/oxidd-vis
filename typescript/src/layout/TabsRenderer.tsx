import React, {FC, ReactPortal, useEffect} from "react";
import {createPortal} from "react-dom";
import {LayoutState} from "./LayoutState";
import {IContentGetter} from "./_types/IContentGetter";
import {useWatch} from "../watchables/react/useWatch";
import {useDerived} from "../watchables/react/useDerived";

/**
 * Makes sure that all the contents are rendered in their target elements
 */
export const TabsRenderer: FC<{state: LayoutState; getContent: IContentGetter}> = ({
    state,
    getContent,
}) => {
    const watch = useWatch();
    const tabData = useDerived(watch =>
        watch(state.allTabs).flatMap(ref => {
            const content = watch(getContent(ref.id));
            if (!content) return [];
            return [{element: ref.element, ...content}];
        })
    );

    return (
        <>
            {watch(tabData).map(({id, element, content}) =>
                createPortal(content, element, id)
            )}
        </>
    );
};
