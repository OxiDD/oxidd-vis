import React, {FC} from "react";
import {IConfigObjectType} from "../../../state/configuration/_types/IConfigObjectType";
import {IntConfig} from "../../../state/configuration/types/IntConfig";
import {IntConfigComp} from "./IntConfigComp";

export const ConfigTypeComp: FC<{value: IConfigObjectType}> = ({value}) => {
    if (value instanceof IntConfig) {
        return <IntConfigComp value={value} />;
    }
    return <></>;
};
