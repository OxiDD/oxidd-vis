import React from "react";
import {DynComp} from "oxidd-vis-rust";
import {NFC} from "../../../utils/_types/NFC";
import {ICompUI} from "../_types/ICompUI";
import {useWatch} from "../../../watchables/react/useWatch";
import {IAriaRef} from "../_types/IAriaRef";

export const DynCompUI: NFC<{
    data: DynComp;
    ChildComp: ICompUI;
    className?: string;
    aria?: IAriaRef;
}> = ({data, ChildComp, className, aria}) => {
    const watch = useWatch();
    const component = watch(data);
    return <ChildComp data={component} className={className} aria={aria} />;
};
