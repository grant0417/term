import React, { useEffect, useRef } from "react";
import "./App.css";
import { Terminal } from "xterm";
import { FitAddon } from "xterm-addon-fit";
import { getCurrent } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";

const current = getCurrent();

function App() {
  const terminalDiv = useRef<HTMLDivElement>(null);

  const term = new Terminal();
  const fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
  term.setOption("fontFamily", "Hack");

  useEffect(() => {
    if (terminalDiv.current) {
      term.open(terminalDiv.current);
      fitAddon.fit();
      term.onData((data) => {
        current.emit("data", data);
      });
      current.emit("ready", "");
    }
  }, []);

  window.addEventListener("resize", () => {
    if (terminalDiv.current) {
      fitAddon.fit();
      current.emit(
        "resize",
        JSON.stringify({
          rows: term.rows,
          cols: term.cols,
        })
      );
    }
  });

  listen("write", (event) => {
    term.write(new Uint8Array([event.payload as number]));
  });

  listen("close", (event) => {
    term.dispose();
    current.close();
  });

  return <div className="terminal" ref={terminalDiv}></div>;
}

export default App;
