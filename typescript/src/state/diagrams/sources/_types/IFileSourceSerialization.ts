export type IFileSourceSerialization = IDddmpData | IBuddyData;
export type IDddmpData = {dddmp: string};
export type IBuddyData = {buddy: {data: string; vars?: string}};
