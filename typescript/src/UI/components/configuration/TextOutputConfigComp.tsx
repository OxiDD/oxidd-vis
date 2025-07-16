import React, {FC, useCallback, useEffect, useState} from "react";
import {TextOutputConfig} from "../../../state/configuration/types/TextOutputConfig";
import {Observer} from "../../../watchables/Observer";
import {copy} from "../../../utils/copy";
import {
    DefaultButton,
    DialogFooter,
    MessageBar,
    MessageBarType,
    PrimaryButton,
    Stack,
    TextField,
} from "@fluentui/react";
import {StyledModal} from "../StyledModal";
import {useWatch} from "../../../watchables/react/useWatch";

export const TextOutputConfigComp: FC<{
    value: TextOutputConfig;
    messageDuration?: number;
}> = ({value, messageDuration}) => {
    const [showTextModal, setShowTextModal] = useState(false);
    const revealTextModal = useCallback(() => setShowTextModal(true), []);
    const hideTextModal = useCallback(() => setShowTextModal(false), []);

    const [copyText, copyStatus] = useCopy();

    const copyOutput = useCallback(() => {
        const output = value.output.get();
        if (!output.text) return;
        copyText(output.text);
    }, []);
    useEffect(() => {
        const obs = new Observer(value.output).add(output => {
            if (!value.autoCopy.get()) return;
            copyOutput();
        });
        return () => obs.destroy();
    }, []);

    const watch = useWatch();
    const output = watch(value.output);

    if (!output.text && !copyStatus) {
        return <></>;
    }
    return (
        <div>
            {copyStatus}
            {output.text && (
                <>
                    <Stack horizontal>
                        <DefaultButton style={{flexGrow: 1}} onClick={revealTextModal}>
                            Show
                        </DefaultButton>
                        <DefaultButton style={{flexGrow: 1}} onClick={copyOutput}>
                            Copy
                        </DefaultButton>
                    </Stack>
                    <TextOutputModal
                        text={output.text}
                        isOpen={showTextModal}
                        onDismiss={hideTextModal}
                    />
                </>
            )}
        </div>
    );
};

export const TextOutputModal: FC<{
    text: string;
    isOpen: boolean;
    onDismiss: () => void;
}> = ({text, isOpen, onDismiss}) => {
    const [copyText, copyStatus] = useCopy();
    const copyOutput = useCallback(() => {
        copyText(text);
    }, [text]);
    return (
        <StyledModal title="Text output" isOpen={isOpen} onDismiss={onDismiss}>
            <TextField value={text} multiline rows={5} />
            {copyStatus}
            <DialogFooter>
                <PrimaryButton onClick={copyOutput}>Copy text</PrimaryButton>
            </DialogFooter>
        </StyledModal>
    );
};

function useCopy(
    messageDuration: number = 5000
): [(text: string) => void, JSX.Element | undefined] {
    const [showCopiedMessage, setShowCopiedMessage] = useState(false);
    const hideCopiedMessage = useCallback(() => setShowCopiedMessage(false), []);
    useEffect(() => {
        if (showCopiedMessage) {
            const timeoutID = setTimeout(hideCopiedMessage, messageDuration);
            return () => clearTimeout(timeoutID);
        }
    }, [showCopiedMessage]);

    const [showErrorMessage, setShowErrorMessage] = useState(false);
    const hideErrorMessage = useCallback(() => setShowErrorMessage(false), []);
    useEffect(() => {
        if (showErrorMessage) {
            const timeoutID = setTimeout(hideErrorMessage, messageDuration);
            return () => clearTimeout(timeoutID);
        }
    }, [showErrorMessage]);

    const copyText = useCallback((text: string) => {
        if (copy(text)) {
            setShowCopiedMessage(true);
        } else {
            setShowErrorMessage(true);
        }
    }, []);

    return [
        copyText,
        showCopiedMessage || showErrorMessage ? (
            <>
                {showCopiedMessage && (
                    <MessageBar
                        onDismiss={hideCopiedMessage}
                        messageBarType={MessageBarType.success}>
                        Text copied!
                    </MessageBar>
                )}
                {showErrorMessage && (
                    <MessageBar
                        onDismiss={hideErrorMessage}
                        messageBarType={MessageBarType.error}>
                        Copy failed!
                    </MessageBar>
                )}
            </>
        ) : undefined,
    ];
}
