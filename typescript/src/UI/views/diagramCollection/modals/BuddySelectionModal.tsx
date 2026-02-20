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
import {InputOption, FileLoader, IFileData} from "./DDDMPSelectionModal";

export const BuddySelectionModal: FC<{
    visible: boolean;
    example: [string, string];
    onSelect: (text: string, vars?: string, name?: string) => void;
    onCancel: () => void;
}> = ({visible, example: [sampleVars, sample], onSelect, onCancel}) => {
    const textRef = useRef<ITextField>(null);
    const varTextRef = useRef<ITextField>(null);
    const [selected, setSelected] = useState<"text" | "file" | "sample">("sample");
    const selectText = useCallback(() => setSelected("text"), []);

    const [fileID, setFileID] = useState(0);
    const [fileTitle, setFileTitle] = useState("");
    const [selectedFileType, setSelectedFileType] = useState("");
    const [textContent, setTextContent] = useState<null | string>(null);
    const [varsTextContent, setVarsTextContent] = useState<null | string>(null);
    const onFileSelect = useCallback((primary: boolean, files: IFileData[]) => {
        setSelected("file");
        if (primary) {
            if (files.length == 1) setSelectedFileType(files[0].type);
        }

        for (const {data, name, type} of files) {
            if (type == "bdd") {
                setFileTitle(name);
                setTextContent(data);
            } else if (type == "bddv") {
                setVarsTextContent(data);
            }
        }
    }, []);
    const onPrimaryFileSelect = useCallback(
        (files: IFileData[]) => onFileSelect(true, files),
        []
    );
    const onSecondaryFileSelect = useCallback(
        (files: IFileData[]) => onFileSelect(false, files),
        []
    );

    const onSubmit = useCallback(() => {
        if (selected == "sample") onSelect(sample, sampleVars);
        else if (selected == "text") {
            const field = textRef.current;
            const varField = varTextRef.current;
            if (field?.value) onSelect(field.value, varField?.value?.trim() || undefined);
        } else {
            if (textContent)
                onSelect(textContent, varsTextContent ?? undefined, fileTitle);
        }
    }, [selected, onSelect, textContent, varsTextContent, fileTitle]);

    useEffect(() => {
        if (!visible) {
            setTimeout(() => {
                setSelected("sample");
                setFileID(id => id + 1);
                setTextContent(null);
                setVarsTextContent(null);
            }, 500);
        }
    }, [visible]);

    return (
        <StyledModal title="Enter Buddy file" isOpen={visible} onDismiss={onCancel}>
            <div className={css({minWidth: 500})}>
                <InputOption
                    name="Text contents"
                    selected={selected == "text"}
                    onSelect={() => setSelected("text")}>
                    <TextField
                        onChange={selectText}
                        multiline
                        rows={selected == "text" ? 5 : 2}
                        componentRef={textRef}
                    />
                    <TextField
                        onChange={selectText}
                        multiline
                        label="optional variable names"
                        rows={selected == "text" ? 5 : 2}
                        componentRef={varTextRef}
                    />
                </InputOption>
                <InputOption
                    name="File selection"
                    selected={selected == "file"}
                    onSelect={() => setSelected("file")}>
                    <FileLoader
                        key={fileID}
                        onLoad={onPrimaryFileSelect}
                        accept=".bdd,.bddv"
                        expanded={selected == "file"}
                    />
                    {selectedFileType == "bdd" && (
                        <FileLoader
                            onLoad={onSecondaryFileSelect}
                            accept=".bddv"
                            expanded={selected == "file"}
                        />
                    )}
                    {selectedFileType == "bddv" && (
                        <FileLoader
                            onLoad={onSecondaryFileSelect}
                            accept=".bdd"
                            expanded={selected == "file"}
                        />
                    )}
                </InputOption>
                <InputOption
                    name="Load example"
                    selected={selected == "sample"}
                    onSelect={() => setSelected("sample")}>
                    <TextField
                        readOnly
                        multiline
                        rows={selected == "sample" ? 5 : 2}
                        defaultValue={sample}
                    />
                    <TextField
                        readOnly
                        multiline
                        label="optional variable names"
                        rows={selected == "sample" ? 5 : 2}
                        defaultValue={sampleVars}
                    />
                </InputOption>
            </div>
            <PrimaryButton
                onClick={onSubmit}
                disabled={selected == "file" && !textContent}>
                Load
            </PrimaryButton>
        </StyledModal>
    );
};
