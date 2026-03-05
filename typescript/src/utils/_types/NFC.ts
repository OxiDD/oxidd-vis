import {ReactElement} from "react";

export type NFC<P = {}> = (props: P) => ReactElement<any, any> | null;
