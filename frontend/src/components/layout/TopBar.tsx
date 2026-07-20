import { Monitor, Wifi, WifiOff, Zap } from 'lucide-react'
import { useNavigate } from '@tanstack/react-router'
import { useWebSocket } from '../../hooks/useWebSocket'
import { useFlashJobs } from '../../api/queries'
import { cn } from '../../lib/utils'

export function TopBar() {
  const { connected } = useWebSocket()
  const { data: flashData } = useFlashJobs()
  const navigate = useNavigate()

  const activeFlashes =
    flashData?.jobs.filter(
      (j) => j.status === 'pending' || j.status === 'flashing'
    ).length ?? 0

  return (
    <header className="h-14 border-b border-border bg-card/50 flex items-center justify-between px-6">
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <Monitor className="w-4 h-4" />
        <span>USB Station v0.1.0</span>
      </div>
      <div className="flex items-center gap-3">
        {activeFlashes > 0 && (
          <button
            onClick={() => navigate({ to: '/flash' })}
            className="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-blue-500/10 text-blue-500 hover:bg-blue-500/20 transition-colors"
          >
            <Zap className="w-3 h-3" />
            {activeFlashes} active
          </button>
        )}
        <div
          className={cn(
            'flex items-center gap-1.5 text-xs',
            connected ? 'text-green-500' : 'text-red-500'
          )}
        >
          {connected ? (
            <Wifi className="w-3.5 h-3.5" />
          ) : (
            <WifiOff className="w-3.5 h-3.5" />
          )}
          {connected ? 'Connected' : 'Disconnected'}
        </div>
      </div>
    </header>
  )
}
