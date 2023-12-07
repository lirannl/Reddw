import Component from "./Component";
import { customElement } from "solid-element";
import "./index.css";

customElement("wallhaven-config", { value: { searchTerms: [] } }, Component);