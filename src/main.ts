import { mount } from "svelte";
import App from "./App.svelte";
import "./styles/theme.css";
import "./styles/app.css";

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
