import { useEffect, useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'

export type WsMessage = {
  type: string
  payload?: Record<string, unknown>
}

export function useWebSocket() {
  const [connected, setConnected] = useState(false)
  const [lastMessage, setLastMessage] = useState<WsMessage | null>(null)
  const queryClient = useQueryClient()

  useEffect(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const ws = new WebSocket(`${protocol}//${window.location.host}/ws`)

    ws.onopen = () => setConnected(true)
    ws.onclose = () => setConnected(false)
    ws.onerror = () => setConnected(false)
    ws.onmessage = (event) => {
      try {
        const msg: WsMessage = JSON.parse(event.data)
        setLastMessage(msg)

        switch (msg.type) {
          case 'flash_progress':
          case 'flash_created':
          case 'flash_completed':
          case 'flash_failed':
          case 'flash_cancelled':
            queryClient.invalidateQueries({ queryKey: ['flash-jobs'] })
            break
          case 'usb_inserted':
          case 'usb_removed':
            queryClient.invalidateQueries({ queryKey: ['usb-devices'] })
            break
          case 'batch_progress':
          case 'batch_completed':
            queryClient.invalidateQueries({ queryKey: ['batches'] })
            break
        }
      } catch {
        // ignore invalid messages
      }
    }

    return () => ws.close()
  }, [queryClient])

  return { connected, lastMessage }
}
