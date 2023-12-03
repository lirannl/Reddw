import Component from "./Component";
import { customElement } from "solid-element";

customElement("wallhaven-config", { value: { searchTerms: [] } }, Component);