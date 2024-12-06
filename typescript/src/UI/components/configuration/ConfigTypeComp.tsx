import React, {FC} from "react";
import {IConfigObjectType} from "../../../state/configuration/_types/IConfigObjectType";
import {IntConfig} from "../../../state/configuration/types/IntConfig";
import {IntConfigComp} from "./IntConfigComp";
import {LabelConfig} from "../../../state/configuration/types/LabelConfig";
import {LabelConfigComp} from "./LabelConfigComp";
import {CompositeConfig} from "../../../state/configuration/types/CompositeConfig";
import {CompositeConfigComp} from "./CompositeConfigComp";
import {ChoiceConfig} from "../../../state/configuration/types/ChoiceConfig";
import {ChoiceConfigComp} from "./ChoiceConfigComp";
import {ButtonConfigComp} from "./ButtonConfigComp";
import {ButtonConfig} from "../../../state/configuration/types/ButtonConfig";

export const ConfigTypeComp: FC<{value: IConfigObjectType}> = ({value}) => {
    if (value instanceof IntConfig) return <IntConfigComp value={value} />;
    if (value instanceof LabelConfig)
        return <LabelConfigComp value={value} ChildComp={ConfigTypeComp} />;
    if (value instanceof CompositeConfig)
        return <CompositeConfigComp value={value} ChildComp={ConfigTypeComp} />;
    if (value instanceof ChoiceConfig) return <ChoiceConfigComp value={value} />;
    if (value instanceof ButtonConfig) return <ButtonConfigComp value={value} />;
    return <></>;
};
