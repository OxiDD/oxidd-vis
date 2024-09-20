import React, {FC, useCallback} from "react";
import {useWatch} from "../../../watchables/react/useWatch";
import {ChoiceConfig} from "../../../state/configuration/types/ChoiceConfig";
import {IDropdownOption} from "@fluentui/react";
import {StyledDropdown} from "../StyledDropdown";

export const ChoiceConfigComp: FC<{value: ChoiceConfig}> = ({value}) => {
    const watch = useWatch();
    const onChange = useCallback(
        (event: unknown, option?: IDropdownOption, index?: number) => {
            if (index != undefined) {
                value.set(index).commit();
            }
        },
        []
    );
    return (
        <StyledDropdown
            selectedKey={watch(value.selectedIndex)}
            onChange={onChange}
            options={watch(value.options).map((option, i) => ({key: i, text: option}))}
        />
    );
};
