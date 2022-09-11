import { lazy, Suspense } from "solid-js";
import Sources from "./Sources";

function App() {
  return <>
    <Suspense fallback={<div>Loading...</div>}>
      {lazy(() => import("./Config"))}</Suspense>
    <Sources />
  </>
}

export default App;
