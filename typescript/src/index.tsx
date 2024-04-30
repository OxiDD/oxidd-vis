import ReactDOM from "react-dom";
import React from "react";
import {greet} from "oxidd-viz-rust";

ReactDOM.render(
    <div onClick={() => greet("john", "cool")}>Hello world</div>,
    document.getElementById("root")
);
