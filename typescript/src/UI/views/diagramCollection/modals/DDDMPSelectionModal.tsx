import React, {ChangeEvent, FC, useCallback, useEffect, useRef, useState} from "react";
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

export const DDDMPSelectionModal: FC<{
    visible: boolean;
    example: string;
    onSelect: (text: string, name?: string) => void;
    onCancel: () => void;
}> = ({visible, example, onSelect, onCancel}) => {
    const textRef = useRef<ITextField>(null);
    const [selected, setSelected] = useState<"text" | "file" | "sample">("sample");
    const selectText = useCallback(() => setSelected("text"), []);

    const [fileLoading, setFileLoading] = useState(false);
    const [fileTitle, setFileTitle] = useState("");
    const [fileContent, setFileContent] = useState<null | string>(null);
    const onFileChange = useCallback(async (event: ChangeEvent<HTMLInputElement>) => {
        setFileLoading(true);
        const file = event.target.files?.[0];
        if (!file) return;

        setFileTitle(file.name);
        setSelected("file");

        const reader = new FileReader();
        reader.readAsText(file);
        reader.onload = () => {
            const result = reader.result;
            setFileLoading(false);
            if (result) setFileContent(result as string);
        };
        reader.onerror = () => setFileLoading(false);
    }, []);
    const onSubmit = useCallback(() => {
        if (selected == "sample") onSelect(example);
        else if (selected == "text") {
            const field = textRef.current;
            if (field?.value) onSelect(field.value);
        } else {
            if (fileContent) onSelect(fileContent, fileTitle);
        }
    }, [selected, onSelect, fileContent, fileTitle, example]);
    useEffect(() => {
        if (!visible) {
            setTimeout(() => {
                setSelected("sample");
                setFileTitle("");
                setFileContent(null);
            }, 500);
        }
    }, [visible]);

    return (
        <StyledModal title="Enter DDDMP file" isOpen={visible} onDismiss={onCancel}>
            <div className={css({minWidth: 500})}>
                <InputOption
                    name="Text contents"
                    selected={selected == "text"}
                    onSelect={() => setSelected("text")}>
                    <TextField
                        onChange={selectText}
                        multiline
                        rows={selected == "text" ? 8 : 2}
                        componentRef={textRef}
                    />
                </InputOption>
                <InputOption
                    name="File selection"
                    selected={selected == "file"}
                    onSelect={() => setSelected("file")}>
                    <div
                        className={css({
                            position: "relative",
                            cursor: "pointer",
                            input: {
                                position: "absolute",
                                cursor: "pointer",
                                zIndex: 1,
                                left: 0,
                                right: 0,
                                top: 0,
                                bottom: 0,
                                opacity: 0,
                            },
                        })}>
                        {!fileLoading && (
                            <input
                                type="file"
                                id="image"
                                name="image"
                                accept=".dddmp"
                                onChange={onFileChange}
                            />
                        )}
                        <div
                            className={css({
                                height: selected == "file" ? 100 : 30,
                                width: "100%",
                                display: "flex",
                                justifyContent: "center",
                                alignItems: "center",
                                fontSize: 30,
                            })}>
                            {fileLoading ? (
                                <Spinner />
                            ) : fileTitle ? (
                                fileTitle
                            ) : (
                                <FontIcon aria-label="Upload" iconName="Upload" />
                            )}
                        </div>
                    </div>
                </InputOption>
                <InputOption
                    name="Load example"
                    selected={selected == "sample"}
                    onSelect={() => setSelected("sample")}>
                    <TextField
                        readOnly
                        multiline
                        rows={selected == "sample" ? 8 : 2}
                        defaultValue={example}
                    />
                </InputOption>
            </div>
            <PrimaryButton
                onClick={onSubmit}
                disabled={selected == "file" && !fileContent}>
                Load
            </PrimaryButton>
        </StyledModal>
    );
};

export const InputOption: FC<{selected: boolean; onSelect: () => void; name: string}> = ({
    children,
    selected,
    onSelect,
    name,
}) => {
    const theme = useTheme();
    return (
        <div
            style={{
                overflow: "hidden",
                backgroundColor: theme.palette.neutralLighterAlt,
                marginBottom: 10,
            }}>
            <div
                onClick={onSelect}
                style={{
                    backgroundColor: theme.palette.neutralLighter,
                    padding: 10,
                    display: "flex",
                    alignItems: "center",
                    gap: 5,
                    fontSize: 16,
                    fontWeight: 600,
                    cursor: "pointer",
                }}>
                <Checkbox checked={selected} />
                {name}
            </div>
            <div
                style={{
                    padding: 10,
                }}>
                {children}
            </div>
        </div>
    );
};
