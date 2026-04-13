import { useEffect, useState } from "react";

// The SBFB Bridge SDK is loaded globally from /sbfb-bridge.js by
// the host shell. We declare the minimal surface we consume to keep
// the type check happy.
declare global {
  interface Window {
    SBFBBridge: new () => {
      submitTask: (payload: Record<string, unknown>) => Promise<unknown>;
      getStorage: (key: string) => Promise<unknown>;
      setStorage: (key: string, value: unknown) => Promise<unknown>;
      onEvent: (name: string, cb: (payload: unknown) => void) => () => void;
      destroy: () => void;
    };
  }
}

export function App() {
  const [output, setOutput] = useState<string>("(cliquez sur le bouton pour voir un resultat)");

  useEffect(() => {
    const bridge = new window.SBFBBridge();
    const unsub = bridge.onEvent("task_result_ready", (payload) => {
      setOutput("Event recu : " + JSON.stringify(payload, null, 2));
    });
    return () => {
      unsub();
      bridge.destroy();
    };
  }, []);

  async function submit() {
    setOutput("Envoi en cours...");
    try {
      const bridge = new window.SBFBBridge();
      const result = await bridge.submitTask({
        prompt: "Bonjour depuis {{PROJECT_NAME}}",
      });
      setOutput("Resultat : " + JSON.stringify(result, null, 2));
      bridge.destroy();
    } catch (e) {
      setOutput("Erreur : " + (e instanceof Error ? e.message : String(e)));
    }
  }

  return (
    <div style={{
      fontFamily: "system-ui, sans-serif",
      background: "#0a0a0f",
      color: "#e5e5e5",
      minHeight: "100vh",
      padding: "2rem",
      display: "flex",
      flexDirection: "column",
      alignItems: "center",
      gap: "1.5rem",
    }}>
      <h1 style={{ margin: 0 }}>{"{{PROJECT_NAME}}"}</h1>
      <p style={{ color: "#999", textAlign: "center", maxWidth: "32rem" }}>
        App SBFB en React. Le bridge postMessage est monte via
        useEffect et utilise submitTask + onEvent.
      </p>
      <button
        onClick={submit}
        style={{
          background: "#34d399",
          color: "#0a0a0f",
          border: 0,
          padding: "0.75rem 1.5rem",
          borderRadius: "0.5rem",
          fontWeight: 600,
          cursor: "pointer",
        }}
      >
        Envoyer une task de demo
      </button>
      <pre style={{
        background: "rgba(255, 255, 255, 0.05)",
        padding: "1rem",
        borderRadius: "0.5rem",
        maxWidth: "40rem",
        width: "100%",
        overflowX: "auto",
      }}>
        {output}
      </pre>
    </div>
  );
}
