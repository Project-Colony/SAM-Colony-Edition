import { mount } from "svelte";
import App from "./App.svelte";
import { loadPreferences } from "./lib/preferences.svelte";
import "./styles/theme.css";
import "./styles/app.css";

// Mount once the theme, text size and language are in effect, so the first
// paint already looks the way the user left it.
void loadPreferences().then(() => mount(App, { target: document.getElementById("app")! }));
