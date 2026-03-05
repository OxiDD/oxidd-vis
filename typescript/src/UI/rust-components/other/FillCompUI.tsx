import React from "react";
import {FillComp} from "oxidd-vis-rust";
import {NFC} from "../../../utils/_types/NFC";
import {IAriaRef} from "../_types/IAriaRef";
import {ICompUI} from "../_types/ICompUI";
import {useWatch} from "../../../watchables/react/useWatch";
import {css} from "@emotion/css";

export const FillCompUI: NFC<{
    data: FillComp;
    ChildComp: ICompUI;
    className?: string;
    aria: IAriaRef;
}> = ({data, ChildComp, className, aria}) => {
    const watch = useWatch();
    const child = watch(data.content);
    const fillWidth = watch(data.full_width);
    const fillHeight = watch(data.full_height);
    return (
        <ChildComp
            data={child}
            className={`${className} ${css({width: fillWidth ? "100%" : undefined, height: fillHeight ? "100%" : undefined})}`}
            aria={aria}
        />
    );
};
