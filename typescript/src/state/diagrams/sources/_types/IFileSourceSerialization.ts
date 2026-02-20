export type IFileSourceSerialization = IDddmpData | IBuddyData;
export type IDddmpData = {dddmp: {data: string; colors?: string}};
export type IBuddyData = {buddy: {data: string; vars?: string}};
