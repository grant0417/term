import React, { useEffect, useRef } from 'react';
import './App.css';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { getCurrent } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'

const current = getCurrent();

function App() {
  const terminalDiv = useRef<HTMLDivElement>(null);
  const term = new Terminal();
  term.setOption('fontFamily', 'Hack');
  const fitAddon = new FitAddon();
  term.loadAddon(fitAddon);

  useEffect(() => {
    if (terminalDiv.current) {
      term.open(terminalDiv.current);
      fitAddon.fit();
      term.onData((data) => {
        current.emit('data', data);
      });
      current.emit('ready', '');
    }
  }, []);

  window.addEventListener('resize', () => {
    if (terminalDiv.current) {
      fitAddon.fit();
    }
  });

  listen('write', event => {
    console.log('write', event);
    term.write(event.payload as string);
  });

  listen('close', event => {
    term.dispose();
    current.close();
  })

  return <div className="terminal" ref={terminalDiv}></div>;
}

export default App;
