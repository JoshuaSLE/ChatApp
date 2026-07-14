import { useEffect, useState } from "react";
import client from "./api/client";

function App() {
  const [health, setHealth] = useState(null);

  useEffect(() => {
    client.get("/health").then((res) => setHealth(res.data));
  }, []);

  return <pre>{JSON.stringify(health, null, 2)}</pre>;
}

export default App;
