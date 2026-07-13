import { BrowserRouter } from "react-router-dom";
import { AppRouter } from "./router";
import { AppBootstrap } from "../components/app/AppBootstrap";

export function App() {
  return (
  <BrowserRouter>
   <AppBootstrap><AppRouter /></AppBootstrap>
    </BrowserRouter>
  );
}
