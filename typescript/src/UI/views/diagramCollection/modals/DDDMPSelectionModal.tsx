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
    onSelect: (text: string, name?: string) => void;
    onCancel: () => void;
}> = ({visible, onSelect, onCancel}) => {
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
        if (selected == "sample") onSelect(sample);
        else if (selected == "text") {
            const field = textRef.current;
            if (field?.value) onSelect(field.value);
        } else {
            if (fileContent) onSelect(fileContent, fileTitle);
        }
    }, [selected, onSelect, fileContent, fileTitle]);
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
                        defaultValue={sample}
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

const sample = `.ver DDDMP-2.0
.mode A
.varinfo 4
.nnodes 213
.nvars 26
.nsuppvars 26
.suppvarnames x11 x1 x9 x15 x25 x4 x23 x14 x12 x7 x5 x16 x18 x21 x2 x10 x8 x20 x3 x24 x13 x22 x6 x17 x26 x19
.orderedvarnames x1 x3 x5 x6 x26 x18 x16 x17 x15 x2 x12 x7 x25 x11 x4 x14 x21 x23 x24 x13 x20 x19 x22 x10 x9 x8
.ids 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25
.permids 13 0 24 8 12 14 17 15 10 11 2 6 5 16 9 23 25 20 1 18 19 22 3 7 4 21
.nroots 1
.rootids 213
.rootnames b02.bench.dimacs
.nodes
1 F 0 0
2 T 0 0
3 25 2 1
4 25 1 2
5 24 4 1
6 24 3 1
7 24 1 3
8 23 1 6
9 23 7 1
10 23 1 7
11 23 5 1
12 22 10 1
13 22 1 9
14 22 11 1
15 22 1 11
16 22 9 1
17 22 8 1
18 21 1 15
19 21 17 1
20 21 15 1
21 21 1 14
22 21 14 1
23 21 13 1
24 21 12 1
25 21 16 1
26 20 1 22
27 20 22 1
28 20 25 1
29 20 24 1
30 20 1 25
31 20 18 1
32 20 23 1
33 20 1 24
34 20 19 1
35 20 20 1
36 20 1 19
37 20 21 1
38 19 36 1
39 19 1 37
40 19 26 1
41 19 1 27
42 19 1 28
43 19 1 29
44 19 30 1
45 19 31 1
46 19 32 1
47 19 33 1
48 19 1 34
49 19 35 1
50 18 1 48
51 18 1 45
52 18 45 1
53 18 1 46
54 18 1 43
55 18 44 1
56 18 1 41
57 18 1 42
58 18 43 1
59 18 49 1
60 18 47 1
61 18 40 1
62 18 39 1
63 18 38 1
64 18 48 1
65 17 54 1
66 17 55 1
67 17 56 1
68 17 57 1
69 17 58 1
70 17 1 59
71 17 60 1
72 17 1 61
73 17 62 1
74 17 1 63
75 17 64 1
76 17 50 1
77 17 51 1
78 17 1 52
79 17 53 1
80 16 72 1
81 16 73 1
82 16 74 1
83 16 75 1
84 16 1 76
85 16 77 1
86 16 78 1
87 16 79 1
88 16 1 65
89 16 66 1
90 16 1 67
91 16 1 68
92 16 69 1
93 16 70 1
94 16 71 1
95 15 90 1
96 15 84 1
97 15 1 85
98 15 1 86
99 15 1 93
100 15 91 1
101 15 92 94
102 15 81 80
103 15 88 1
104 15 83 82
105 15 1 87
106 15 81 89
107 14 104 1
108 14 1 105
109 14 106 1
110 14 1 95
111 14 1 96
112 14 1 97
113 14 1 98
114 14 1 99
115 14 1 100
116 14 101 1
117 14 102 1
118 14 1 103
119 13 1 117
120 13 118 1
121 13 114 107
122 13 108 1
123 13 1 109
124 13 110 1
125 13 115 1
126 13 111 1
127 13 113 1
128 13 112 1
129 13 1 116
130 12 1 125
131 12 128 1
132 12 127 1
133 12 1 124
134 12 129 1
135 12 119 1
136 12 1 120
137 12 121 1
138 12 122 1
139 12 123 1
140 12 1 126
141 11 138 1
142 11 139 1
143 11 140 1
144 11 130 1
145 11 1 131
146 11 133 132
147 11 134 1
148 11 135 1
149 11 136 1
150 11 137 1
151 10 1 148
152 10 149 1
153 10 150 1
154 10 141 1
155 10 1 142
156 10 143 1
157 10 1 144
158 10 1 145
159 10 1 146
160 10 147 1
161 9 158 1
162 9 1 160
163 9 1 154
164 9 155 152
165 9 1 153
166 9 151 1
167 9 159 156
168 9 157 1
169 8 1 167
170 8 168 1
171 8 161 1
172 8 162 1
173 8 163 1
174 8 164 1
175 8 1 166
176 8 1 165
177 7 175 1
178 7 174 1
179 7 176 1
180 7 173 1
181 7 169 1
182 7 1 170
183 7 1 171
184 7 172 1
185 6 179 1
186 6 182 178
187 6 1 180
188 6 184 1
189 6 1 181
190 6 1 177
191 6 183 1
192 5 189 1
193 5 190 1
194 5 191 1
195 5 1 185
196 5 186 1
197 5 187 188
198 4 193 1
199 4 194 1
200 4 1 195
201 4 196 197
202 4 192 1
203 3 201 1
204 3 1 202
205 3 1 198
206 3 199 1
207 3 200 1
208 2 204 207
209 2 205 1
210 2 206 203
211 1 208 210
212 1 209 1
213 0 212 211
.end`;
