import {invoke} from '@tauri-apps/api/core'

import "./StartStopButton.css"


export const StartStopButton = () => {
  return (
    // <div>StartStopButton</div>
    <div className="button_container">
        <button onClick={() => invoke('start_collect')}>Start</button>
        <button onClick={() => invoke('stop_collect')}>Stop</button>
    </div>
    
  )
}
