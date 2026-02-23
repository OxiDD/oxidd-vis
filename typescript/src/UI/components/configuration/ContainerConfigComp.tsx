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
                ...(margin.left ? {marginLeft: margin.left} : undefined),
                ...(margin.right ? {marginRight: margin.right} : undefined),
                ...(margin.top ? {marginTop: margin.top} : undefined),
                ...(margin.bottom ? {marginBottom: margin.bottom} : undefined),
            }}>
            <ChildComp value={watch(value.value)} />
        </div>
    );
};
