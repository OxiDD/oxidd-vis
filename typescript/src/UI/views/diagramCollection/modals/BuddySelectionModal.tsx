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
import {InputOption} from "./DDDMPSelectionModal";

export const BuddySelectionModal: FC<{
    visible: boolean;
    onSelect: (text: string, vars?: string, name?: string) => void;
    onCancel: () => void;
}> = ({visible, onSelect, onCancel}) => {
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
            if (type == "out") {
                setFileTitle(name);
                setTextContent(data);
            } else if (type == "outv") {
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
                        readOnly
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
                        accept=".out,.outv"
                        expanded={selected == "file"}
                    />
                    {selectedFileType == "out" && (
                        <FileLoader
                            onLoad={onSecondaryFileSelect}
                            accept=".outv"
                            expanded={selected == "file"}
                        />
                    )}
                    {selectedFileType == "outv" && (
                        <FileLoader
                            onLoad={onSecondaryFileSelect}
                            accept=".out"
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

type IFileData = {data: string; name: string; type: string};

const FileLoader: FC<{
    onLoad: (data: IFileData[]) => void;
    accept: string;
    expanded: boolean;
}> = ({onLoad, expanded, accept}) => {
    const [fileLoading, setFileLoading] = useState(false);
    const [fileTitles, setFileTitles] = useState<string[]>([]);
    const onFileChange = useCallback(async (event: ChangeEvent<HTMLInputElement>) => {
        setFileLoading(true);
        const rawFiles = event.target.files;
        if (!rawFiles || rawFiles.length == 0) return;
        const files = [...rawFiles];

        setFileTitles(files.map(file => file.name));

        const textPromises = files.map(
            file =>
                new Promise<{data: string; name: string; type: string}>((res, rej) => {
                    const reader = new FileReader();
                    reader.readAsText(file);
                    reader.onload = () => {
                        const result = reader.result;
                        const nameParts = file.name.split(".");
                        res({
                            data: result as string,
                            name: file.name,
                            type: nameParts[nameParts.length - 1],
                        });
                    };
                    reader.onerror = rej;
                })
        );

        Promise.all(textPromises)
            .then(data => {
                setFileLoading(false);
                onLoad(data);
            })
            .catch(() => {
                setFileLoading(false);
            });
    }, []);

    return (
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
                    multiple
                    accept={accept}
                    onChange={onFileChange}
                />
            )}
            <div
                className={css({
                    height: expanded ? 100 : 30,
                    width: "100%",
                    display: "flex",
                    justifyContent: "center",
                    alignItems: "center",
                    flexDirection: "column",
                    fontSize: 30,
                })}>
                {fileLoading ? (
                    <Spinner />
                ) : fileTitles.length > 0 ? (
                    fileTitles.map((t, i) => <div key={i}>{t}</div>)
                ) : (
                    <FontIcon aria-label="Upload" iconName="Upload" />
                )}
            </div>
        </div>
    );
};

const sample = `7 30
24 10 22 13 11 12 26 28 0 16 5 17 3 29 6 4 15 20 8 9 21 23 7 1 2 25 18 27 19 14 
28 13 0 1
72 7 1 28
131 6 0 72
29 13 1 0
313 7 0 29
145 6 313 1
135 4 131 145`;
const sampleVars = `IN_R_REG_6__SCAN_IN
IN_R_REG_4__SCAN_IN
IN_R_REG_0__SCAN_IN
MAR_REG_2__SCAN_IN
I_3_
I_2_
IN_R_REG_3__SCAN_IN
STATO_REG_0__SCAN_IN
OUT_R_REG_3__SCAN_IN
I_4_
O_REG_2__SCAN_IN
I_5_
OUT_R_REG_0__SCAN_IN
STATO_REG_1__SCAN_IN
O_REG_1__SCAN_IN
O_REG_3__SCAN_IN
IN_R_REG_7__SCAN_IN
I_0_
START
MAR_REG_1__SCAN_IN
I_1_
IN_R_REG_1__SCAN_IN
O_REG_0__SCAN_IN
OUT_R_REG_2__SCAN_IN
OUT_R_REG_1__SCAN_IN
IN_R_REG_2__SCAN_IN
I_7_
MAR_REG_0__SCAN_IN
I_6_
IN_R_REG_5__SCAN_IN`;
