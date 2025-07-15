import React, {FC} from "react";
import {useWatch} from "../../../watchables/react/useWatch";
import {IConfigObjectType} from "../../../state/configuration/_types/IConfigObjectType";
import {ContainerConfig} from "../../../state/configuration/types/ContainerConfig";

export const ContainerConfigComp: FC<{
    value: ContainerConfig;
    ChildComp: FC<{value: IConfigObjectType}>;
}> = ({value, ChildComp}) => {
    const watch = useWatch();
    const margin = watch(value.margin);
    if (watch(value.hidden)) {
        return <></>;
    }

    return (
        <div
            style={{
                marginLeft: margin.left,
                marginRight: margin.right,
                marginTop: margin.top,
                marginBottom: margin.bottom,
            }}>
            <ChildComp value={watch(value.value)} />
        </div>
    );
};
