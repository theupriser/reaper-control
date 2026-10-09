import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { applyAppearance, appearanceOf, defaultAppearance } from "./lib/appearance";
import { currentSettings } from "./lib/ipc";

applyAppearance(document.documentElement, defaultAppearance);
currentSettings()
  .then((view) => applyAppearance(document.documentElement, appearanceOf(view.settings.appearance)))
  .catch(() => {});

export default mount(App, { target: document.getElementById("app")! });
