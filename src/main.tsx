import ReactDOM from "react-dom/client";
import App from "./App";

async function renderApp() {
  await Promise.allSettled([
    document.fonts.load('400 13px "CLI Hub JetBrainsMono NF"'),
    document.fonts.load('700 13px "CLI Hub JetBrainsMono NF"'),
  ]);

  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <App />,
  );
}

void renderApp();
