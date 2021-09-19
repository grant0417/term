import React, { useEffect, useRef } from 'react';
import './App.css';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { getCurrent } from '@tauri-apps/api/window'

const current = getCurrent();

function App() {
  const terminalDiv = useRef<HTMLDivElement>(null);
  const term = new Terminal();
  const fitAddon = new FitAddon();
  term.loadAddon(fitAddon);

  useEffect(() => {
    if (terminalDiv.current) {
      term.open(terminalDiv.current);
      term.write('Hello from \x1B[1;3;31mxterm.js\x1B[0m $\r\n');
      fitAddon.fit();
      term.onData((data) => {
        current.emit('data', data);
        term.write(data);
      });
    }
  }, []);

  window.addEventListener('resize', () => {
    if (terminalDiv.current) {
      fitAddon.fit();
    }
  });

  return <div className="terminal" ref={terminalDiv}></div>;
}

export default App;
