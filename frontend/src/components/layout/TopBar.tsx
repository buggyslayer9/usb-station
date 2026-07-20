import { Monitor, Wifi, WifiOff } from 'lucide-react'
import { useEffect, useState } from 'react'

export function TopBar() {
  const [connected, setConnected] = useState(false)

  useEffect(() => {
    const ws = new WebSocket(`ws://${window.location.host}/ws`)
    ws.onopen = () => setConnected(true)
    ws.onclose = () => setConnected(false)
    return () => ws.close()
  }, [])

  return (
    <header className="h-14 border-b border-border bg-card/50 flex items-center justify-between px-6">
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <Monitor className="w-4 h-4" />
        <span>USB Station v0.1.0</span>
      </div>
      <div className="flex items-center gap-2">
        <div className={`flex items-center gap-1.5 text-xs ${connected ? 'text-green-500' : 'text-red-500'}`}>
          {connected ? <Wifi className="w-3.5 h-3.5" /> : <WifiOff className="w-3.5 h-3.5" />}
          {connected ? 'Connected' : 'Disconnected'}
        </div>
      </div>
    </header>
  )
}
