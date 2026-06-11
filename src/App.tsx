function App() {
  return (
    <div
      style={{
        minHeight: "100vh",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        fontFamily: "system-ui, -apple-system, sans-serif",
        backgroundColor: "#f5f5f5",
        color: "#333",
      }}
    >
      <h1 style={{ marginBottom: "12px", fontSize: "2rem" }}>
        fpga-flow-mind
      </h1>
      <p
        style={{
          marginBottom: "32px",
          fontSize: "1.1rem",
          color: "#666",
        }}
      >
        Phase 1：Workspace 扫描与阶段识别
      </p>
      <button
        disabled
        style={{
          padding: "10px 24px",
          fontSize: "1rem",
          borderRadius: "6px",
          border: "1px solid #ccc",
          backgroundColor: "#e0e0e0",
          color: "#999",
          cursor: "not-allowed",
        }}
      >
        打开项目
      </button>
    </div>
  );
}

export default App;
