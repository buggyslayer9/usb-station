import { motion } from 'framer-motion'
import { Database, Disc3, Zap, Layers } from 'lucide-react'
import { Card, CardHeader, CardTitle, CardContent } from '../ui/card'
import { useUsbDevices, useIsoImages } from '../../api/queries'
import { formatBytes } from '../../lib/utils'

export function DashboardPage() {
  const { data: usbData } = useUsbDevices()
  const { data: isoData } = useIsoImages()

  const stats = [
    { label: 'USB Devices', value: usbData?.devices.length ?? 0, icon: Database },
    { label: 'ISO Images', value: isoData?.images.length ?? 0, icon: Disc3 },
    { label: 'Active Flashes', value: 0, icon: Zap },
    { label: 'Batches', value: 0, icon: Layers },
  ]

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold tracking-tight">Dashboard</h2>
        <p className="text-muted-foreground mt-1">Overview of your USB Station</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {stats.map((stat, i) => (
          <motion.div
            key={stat.label}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.1 }}
          >
            <Card className="glass">
              <CardHeader className="flex flex-row items-center justify-between pb-2">
                <CardTitle className="text-sm font-medium text-muted-foreground">
                  {stat.label}
                </CardTitle>
                <stat.icon className="w-4 h-4 text-primary" />
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-bold">{stat.value}</div>
              </CardContent>
            </Card>
          </motion.div>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card className="glass">
          <CardHeader>
            <CardTitle>Connected USB Devices</CardTitle>
          </CardHeader>
          <CardContent>
            {usbData?.devices.length === 0 ? (
              <p className="text-muted-foreground text-sm">No USB devices detected</p>
            ) : (
              <div className="space-y-2">
                {usbData?.devices.slice(0, 5).map((d) => (
                  <div key={d.id} className="flex justify-between text-sm py-1">
                    <span className="font-mono text-xs">{d.device_path}</span>
                    <span className="text-muted-foreground">{d.model ?? d.vendor ?? 'Unknown'}</span>
                    <span>{formatBytes(d.capacity_bytes)}</span>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        <Card className="glass">
          <CardHeader>
            <CardTitle>Recent ISO Library</CardTitle>
          </CardHeader>
          <CardContent>
            {isoData?.images.length === 0 ? (
              <p className="text-muted-foreground text-sm">No ISOs found. Add some to /storage/iso</p>
            ) : (
              <div className="space-y-2">
                {isoData?.images.slice(0, 5).map((img) => (
                  <div key={img.id} className="flex justify-between text-sm py-1">
                    <span className="truncate max-w-[200px]">{img.filename}</span>
                    <span className="text-muted-foreground">{img.detected_os ?? 'Unknown'}</span>
                    <span>{formatBytes(img.file_size_bytes)}</span>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
