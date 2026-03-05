import {BinaryInputComp} from "oxidd-vis-rust";
import React, {ChangeEvent, useCallback, useState} from "react";
import {useWatch} from "../../../watchables/react/useWatch";
import {Checkbox, FontIcon, Spinner} from "@fluentui/react";
import {NFC} from "../../../utils/_types/NFC";
import {IAriaRef} from "../_types/IAriaRef";
import {css} from "@emotion/css";

export const BinaryInputCompUI: NFC<{
    data: BinaryInputComp;
    editData?: BinaryInputComp;
    className?: string;
    aria?: IAriaRef;
}> = ({data, editData = data, className, aria}) => {
    const watch = useWatch();
    const input = data.data;
    const filename = watch(input.filename);
    const disabled = watch(data.disabled);

    const [fileLoading, setFileLoading] = useState(false);

    const onFileChange = useCallback(
        async (event: ChangeEvent<HTMLInputElement>) => {
            setFileLoading(true);
            const file = event.target.files?.[0];
            if (!file) return;

            input.filename.set(file.name).commit();

            const reader = new FileReader();
            reader.readAsArrayBuffer(file);
            reader.onload = () => {
                const result = reader.result;
                setFileLoading(false);
                if (result) {
                    const uint8Array = new Uint8Array(result as ArrayBuffer);
                    input.set(uint8Array).commit();
                }
            };
            reader.onerror = () => setFileLoading(false);
        },
        [input]
    );

    return (
        <div
            aria-describedby={aria?.descriptionID}
            aria-labelledby={aria?.labelID}
            className={`${className} ${css({
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
                ...(disabled ? {opacity: 0.5, cursor: "not-allowed"} : undefined),
            })}`}>
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
                    minHeight: 50,
                    width: "100%",
                    display: "flex",
                    justifyContent: "center",
                    alignItems: "center",
                    fontSize: 30,
                })}>
                {fileLoading ? (
                    <Spinner />
                ) : filename ? (
                    filename
                ) : (
                    <FontIcon aria-label="Upload" iconName="Upload" />
                )}
            </div>
        </div>
    );
};
