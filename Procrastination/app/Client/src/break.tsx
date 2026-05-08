import ReactDOM from "react-dom/client";
import "./global.css";
import { BreakWindow } from "./Components/BreakWindow";

const savedTheme = localStorage.getItem("theme") ?? "dark";
document.documentElement.setAttribute("data-theme", savedTheme);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<BreakWindow />);
