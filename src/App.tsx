export default function App() {
  return (
    <main
      style={{
        alignItems: "center",
        color: "#f5f7fb",
        display: "grid",
        fontFamily: "system-ui, sans-serif",
        minHeight: "100vh",
        placeItems: "center",
        background: "linear-gradient(135deg, #111827 0%, #1f2937 100%)",
      }}
    >
      <section style={{ textAlign: "center" }}>
        <p style={{ letterSpacing: "0.2em", textTransform: "uppercase" }}>
          Push Deck
        </p>
        <h1 style={{ fontSize: "3rem", margin: 0 }}>Workspace scaffold ready</h1>
        <p style={{ color: "#cbd5e1" }}>
          Tauri shell and Vite frontend are wired for the next task.
        </p>
      </section>
    </main>
  );
}
