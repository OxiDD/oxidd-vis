import React, {FC, forwardRef} from "react";
import {ISpinButtonProps, SpinButton} from "@fluentui/react";
import {css} from "@emotion/css";

export const StyledSpinButton: FC<ISpinButtonProps> = forwardRef((props, ref) => (
    <SpinButton
        ref={ref}
        styles={{
            input: {width: "100%", minWidth: 0},
            spinButtonWrapper: {minWidth: 0},
            root: {minWidth: 0},
        }}
        className={`${props.className} ${css({
            ">*:after": {border: 0},
        })}`}
        {...props}
    />
));
