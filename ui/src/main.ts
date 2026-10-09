import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { applyAppearance, defaultAppearance } from "./lib/appearance";

applyAppearance(document.documentElement, defaultAppearance);

export default mount(App, { target: document.getElementById("app")! });
