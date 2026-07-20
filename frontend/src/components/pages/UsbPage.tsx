import { useState } from 'react'
import { motion } from 'framer-motion'
import {
  HardDrive,
  AlertTriangle,
  CheckCircle,
  XCircle,
  LogOut,
  Loader2,
} from 'lucide-react'
import * as Toast from '@radix-ui/react-toast'
import { Card, CardHeader, CardTitle, CardContent } from '../ui/card'
import { useUsbDevices, useEjectUsb } from '../../api/queries'
import { cn, formatBytes } from '../../lib/utils'

export function UsbPage() {
  const { data, isLoading } = useUsbDevices()
  const ejectUsb = useEjectUsb()
  const [toastOpen, setToastOpen] = useState(false)
  const [toastData, setToastData] = useState({
    title: '',
    description: '',
    variant: 'success' as 'success' | 'error',
  })

  const handleEject = (id: string) => {
    ejectUsb.mutate(id, {
      onSuccess: () => {
        setToastData({
          title: 'Ejected',
          description: 'USB device safely ejected',
          variant: 'success',
        })
        setToastOpen(true)
      },
      onError: (err) => {
        setToastData({
          title: 'Eject Failed',
          description:
            err instanceof Error ? err.message : 'An unknown error occurred',
          variant: 'error',
        })
        setToastOpen(true)
      },
    })
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold tracking-tight">USB Devices</h2>
        <p className="text-muted-foreground mt-1">
          Detected USB storage devices
        </p>
      </div>

      {isLoading && (
        <p className="text-muted-foreground">Scanning devices...</p>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {data?.devices.map((device, i) => (
          <motion.div
            key={device.id}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.05 }}
          >
            <Card className="glass">
              <CardHeader className="flex flex-row items-center justify-between">
                <div className="flex items-center gap-3">
                  <HardDrive className="w-8 h-8 text-primary" />
                  <div>
                    <CardTitle className="text-base">
                      {device.model || device.vendor || 'Unknown Device'}
                    </CardTitle>
                    <p className="text-xs text-muted-foreground font-mono">
                      {device.device_path}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-1">
                  {device.health === 'Good' ? (
                    <CheckCircle className="w-4 h-4 text-green-500" />
                  ) : device.health === 'Warning' ? (
                    <AlertTriangle className="w-4 h-4 text-yellow-500" />
                  ) : (
                    <XCircle className="w-4 h-4 text-red-500" />
                  )}
                </div>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span className="text-muted-foreground">Capacity</span>
                    <p className="font-medium">
                      {formatBytes(device.capacity_bytes)}
                    </p>
                  </div>
                  <div>
                    <span className="text-muted-foreground">Serial</span>
                    <p className="font-medium font-mono text-xs">
                      {device.serial ?? 'N/A'}
                    </p>
                  </div>
                  <div>
                    <span className="text-muted-foreground">Mounted</span>
                    <p className="font-medium">
                      {device.is_mounted ? 'Yes' : 'No'}
                    </p>
                  </div>
                  <div>
                    <span className="text-muted-foreground">System Disk</span>
                    <p className="font-medium">
                      {device.is_system_disk ? 'Yes' : 'No'}
                    </p>
                  </div>
                </div>
                {device.is_system_disk && (
                  <div className="mt-3 flex items-center gap-2 text-xs text-red-500 bg-red-500/10 p-2 rounded">
                    <AlertTriangle className="w-3 h-3" />
                    System disk — flashing is blocked for safety
                  </div>
                )}
                {!device.is_system_disk && (
                  <button
                    onClick={() => handleEject(device.id)}
                    disabled={ejectUsb.isPending}
                    className={cn(
                      'mt-3 flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors w-full justify-center',
                      ejectUsb.isPending
                        ? 'bg-muted text-muted-foreground cursor-not-allowed'
                        : 'bg-yellow-500/10 text-yellow-600 hover:bg-yellow-500/20'
                    )}
                  >
                    {ejectUsb.isPending ? (
                      <Loader2 className="w-3.5 h-3.5 animate-spin" />
                    ) : (
                      <LogOut className="w-3.5 h-3.5" />
                    )}
                    {ejectUsb.isPending ? 'Ejecting...' : 'Safely Eject'}
                  </button>
                )}
              </CardContent>
            </Card>
          </motion.div>
        ))}
        {data?.devices.length === 0 && !isLoading && (
          <div className="col-span-full text-center py-12 text-muted-foreground">
            No USB devices detected. Plug one in and it will appear here
            automatically.
          </div>
        )}
      </div>

      <Toast.Root
        open={toastOpen}
        onOpenChange={setToastOpen}
        className={cn(
          'rounded-lg border px-4 py-3 shadow-lg',
          toastData.variant === 'success'
            ? 'bg-green-500/10 border-green-500/20 text-green-600'
            : 'bg-red-500/10 border-red-500/20 text-red-600'
        )}
      >
        <Toast.Title className="text-sm font-medium">
          {toastData.title}
        </Toast.Title>
        <Toast.Description className="text-xs mt-1">
          {toastData.description}
        </Toast.Description>
      </Toast.Root>
    </div>
  )
}
