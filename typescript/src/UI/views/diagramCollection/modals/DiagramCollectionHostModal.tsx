import React, {
    ChangeEvent,
    FC,
    KeyboardEvent,
    useCallback,
    useEffect,
    useRef,
    useState,
} from "react";
import {StyledModal} from "../../../components/StyledModal";
import {
    Checkbox,
    FontIcon,
    ITextField,
    PrimaryButton,
    Spinner,
    TextField,
    useTheme,
} from "@fluentui/react";
import {css} from "@emotion/css";

export const DiagramCollectionHostModal: FC<{
    visible: boolean;
    onSelect: (host: string) => void;
    onCancel: () => void;
}> = ({visible, onSelect, onCancel}) => {
    const [host, setHost] = useState("http://localhost:4000");
    const theme = useTheme();

    const onSubmit = useCallback(() => {
        onSelect(host);
    }, [host, onSelect]);
    const onKeyDown = useCallback(
        (event: KeyboardEvent<unknown>) => {
            if (event.key == "Enter") onSubmit();
        },
        [onSubmit]
    );
    return (
        <StyledModal title="Enter host address" isOpen={visible} onDismiss={onCancel}>
            <TextField
                value={host}
                onChange={(e, v) => v && setHost(v)}
                label="Host"
                styles={{root: {marginBottom: theme.spacing.m}}}
                onKeyDown={onKeyDown}
            />
            <PrimaryButton onClick={onSubmit} disabled={!host}>
                Add
            </PrimaryButton>
        </StyledModal>
    );
};
