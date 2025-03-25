export type IDiagramCollectionType = "manual" | "remote-http";
export type IDiagramCollectionConfig =
    | {
          type: "manual";
      }
    | {
          type: "remote-http";
          url: string;
      };
