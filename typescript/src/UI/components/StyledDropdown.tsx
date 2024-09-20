import React, {FC, forwardRef} from "react";
import {Dropdown, IDropdownProps} from "@fluentui/react";
import {css} from "@emotion/css";

export const StyledDropdown: FC<IDropdownProps> = forwardRef((props, ref) => (
    <Dropdown
        ref={ref}
        className={`${props.className} ${css({
            ".ms-Dropdown-title": {border: 0},
        })}`}
        {...props}
    />
));
