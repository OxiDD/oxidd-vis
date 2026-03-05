import {IAriaRef} from "./_types/IAriaRef";

/**
 * Adds a label ID to the aria
 * @param labelID The label id to add
 * @param aria The current aria
 * @returns The new aria
 */
export function addAriaLabel(labelID: string, aria: IAriaRef): IAriaRef {
    return {
        ...aria,
        labelID: aria?.labelID ? `${aria.labelID} ${labelID}` : labelID,
    };
}
/**
 * Retrieves the label from the aria
 * @param aria The aria to take from
 * @returns The remaining aria and the label
 */
export function takeAriaLabel(aria: IAriaRef): [IAriaRef, string | undefined] {
    return [
        aria?.descriptionID ? {descriptionID: aria.descriptionID} : undefined,
        aria?.labelID,
    ];
}

/**
 * Adds a description ID to the aria
 * @param descriptionID The description id to add
 * @param aria The current aria
 * @returns The new aria
 */
export function addAriaDescription(descriptionID: string, aria: IAriaRef): IAriaRef {
    return {
        ...aria,
        descriptionID: aria?.descriptionID
            ? `${aria.descriptionID} ${descriptionID}`
            : descriptionID,
    };
}
/**
 * Retrieves the description from the aria
 * @param aria The aria to take from
 * @returns The remaining aria and the description
 */
export function takeAriaDescription(aria: IAriaRef): [IAriaRef, string | undefined] {
    return [aria?.labelID ? {labelID: aria.labelID} : undefined, aria?.descriptionID];
}
