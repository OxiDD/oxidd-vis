export type IStorage = {
    /** Saves the given data */
    save(data: string): void;
    /** Loads teh data */
    load(): string | undefined;
};
