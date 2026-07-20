import { motion } from 'framer-motion'
import { HardDrive, AlertTriangle, CheckCircle, XCircle } from 'lucide-react'
import { Card, CardHeader, CardTitle, CardContent } from '../ui/card'
import { useUsbDevices } from '../../api/queries'
import { formatBytes } from '../../lib/utils'

export function UsbPage() {
  const { data, isLoading } = useUsbDevices()

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold tracking-tight">USB Devices</h2>
        <p className="text-muted-foreground mt-1">Detected USB storage devices</p>
      </div>

      {isLoading && <p className="text-muted-foreground">Scanning devices...</p>}

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
                    <p className="font-medium">{formatBytes(device.capacity_bytes)}</p>
                  </div>
                  <div>
                    <span className="text-muted-foreground">Serial</span>
                    <p className="font-medium font-mono text-xs">
                      {device.serial ?? 'N/A'}
                    </p>
                  </div>
                  <div>
                    <span className="text-muted-foreground">Mounted</span>
                    <p className="font-medium">{device.is_mounted ? 'Yes' : 'No'}</p>
                  </div>
                  <div>
                    <span className="text-muted-foreground">System Disk</span>
                    <p className="font-medium">{device.is_system_disk ? '⚠️ Yes' : 'No'}</p>
                  </div>
                </div>
                {device.is_system_disk && (
                  <div className="mt-3 flex items-center gap-2 text-xs text-red-500 bg-red-500/10 p-2 rounded">
                    <AlertTriangle className="w-3 h-3" />
                    System disk — flashing is blocked for safety
                  </div>
                )}
              </CardContent>
            </Card>
          </motion.div>
        ))}
        {data?.devices.length === 0 && !isLoading && (
          <div className="col-span-full text-center py-12 text-muted-foreground">
            No USB devices detected. Plug one in and it will appear here automatically.
          </div>
        )}
      </div>
    </div>
  )
}
