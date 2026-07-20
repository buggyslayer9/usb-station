import { motion } from 'framer-motion'
import {
  Database,
  Disc3,
  Zap,
  Layers,
  CheckCircle,
  XCircle,
  Clock,
} from 'lucide-react'
import { Card, CardHeader, CardTitle, CardContent } from '../ui/card'
import {
  useUsbDevices,
  useIsoImages,
  useFlashJobs,
  useSystemInfo,
} from '../../api/queries'
import { useWebSocket } from '../../hooks/useWebSocket'
import { cn, formatBytes, formatEta } from '../../lib/utils'

export function DashboardPage() {
  const { data: usbData } = useUsbDevices()
  const { data: isoData } = useIsoImages()
  const { data: flashData } = useFlashJobs()
  const { data: sysInfo } = useSystemInfo()
  const { connected } = useWebSocket()

  const activeFlashes =
    flashData?.jobs.filter(
      (j) => j.status === 'pending' || j.status === 'flashing'
    ).length ?? 0

  const recentJobs = [...(flashData?.jobs ?? [])]
    .sort((a, b) => (a.id > b.id ? -1 : 1))
    .slice(0, 5)

  const stats = [
    { label: 'USB Devices', value: usbData?.devices.length ?? 0, icon: Database },
    { label: 'ISO Images', value: isoData?.images.length ?? 0, icon: Disc3 },
    { label: 'Active Flashes', value: activeFlashes, icon: Zap },
    { label: 'Batches', value: 0, icon: Layers },
  ]

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-2">
        <div
          className={cn(
            'w-2 h-2 rounded-full',
            connected ? 'bg-green-500' : 'bg-red-500'
          )}
        />
        <span className="text-xs text-muted-foreground">
          {connected ? 'Live' : 'Offline'}
        </span>
      </div>

      <div>
        <h2 className="text-3xl font-bold tracking-tight">Dashboard</h2>
        <p className="text-muted-foreground mt-1">
          Overview of your USB Station
        </p>
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
              <p className="text-muted-foreground text-sm">
                No USB devices detected
              </p>
            ) : (
              <div className="space-y-2">
                {usbData?.devices.slice(0, 5).map((d) => (
                  <div key={d.id} className="flex justify-between text-sm py-1">
                    <span className="font-mono text-xs">{d.device_path}</span>
                    <span className="text-muted-foreground">
                      {d.model ?? d.vendor ?? 'Unknown'}
                    </span>
                    <span>{formatBytes(d.capacity_bytes)}</span>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        <Card className="glass">
          <CardHeader>
            <CardTitle>Recent Flashes</CardTitle>
          </CardHeader>
          <CardContent>
            {recentJobs.length === 0 ? (
              <p className="text-muted-foreground text-sm">
                No flash activity yet
              </p>
            ) : (
              <div className="space-y-2">
                {recentJobs.map((job) => (
                  <div
                    key={job.id}
                    className="flex items-center justify-between text-sm py-1"
                  >
                    <div className="flex items-center gap-2">
                      {job.status === 'completed' ? (
                        <CheckCircle className="w-3.5 h-3.5 text-green-500" />
                      ) : job.status === 'failed' ? (
                        <XCircle className="w-3.5 h-3.5 text-red-500" />
                      ) : (
                        <Clock className="w-3.5 h-3.5 text-blue-500" />
                      )}
                      <span>{job.status}</span>
                    </div>
                    <span
                      className={cn(
                        'text-xs font-medium',
                        job.status === 'completed' && 'text-green-500',
                        job.status === 'failed' && 'text-red-500'
                      )}
                    >
                      {job.status === 'completed'
                        ? '100%'
                        : `${job.progress_percent}%`}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {sysInfo && (
          <Card className="glass lg:col-span-2">
            <CardHeader>
              <CardTitle>System</CardTitle>
            </CardHeader>
            <CardContent className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div>
                <span className="text-sm text-muted-foreground">
                  Disk Usage
                </span>
                <p className="text-lg font-medium">
                  {sysInfo.disk_usage_percent.toFixed(1)}%
                </p>
              </div>
              <div>
                <span className="text-sm text-muted-foreground">Storage</span>
                <p className="text-lg font-medium">
                  {formatBytes(sysInfo.used_bytes)} /{' '}
                  {formatBytes(sysInfo.total_capacity_bytes)}
                </p>
              </div>
              <div>
                <span className="text-sm text-muted-foreground">Uptime</span>
                <p className="text-lg font-medium">
                  {formatEta(sysInfo.uptime_seconds)}
                </p>
              </div>
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  )
}
