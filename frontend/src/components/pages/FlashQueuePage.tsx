import { motion } from 'framer-motion'
import { Zap, XCircle, CheckCircle, Loader2 } from 'lucide-react'
import * as Progress from '@radix-ui/react-progress'
import { Card, CardHeader, CardTitle, CardContent } from '../ui/card'
import { useFlashJobs, useUsbDevices, useIsoImages, useCancelFlash } from '../../api/queries'
import { cn, formatSpeed, formatEta } from '../../lib/utils'

function StatusBadge({ status }: { status: string }) {
  const colors: Record<string, string> = {
    completed: 'bg-green-500/10 text-green-500 border-green-500/20',
    failed: 'bg-red-500/10 text-red-500 border-red-500/20',
    flashing: 'bg-blue-500/10 text-blue-500 border-blue-500/20',
    pending: 'bg-yellow-500/10 text-yellow-500 border-yellow-500/20',
    cancelled: 'bg-gray-500/10 text-gray-500 border-gray-500/20',
  }

  return (
    <span
      className={cn(
        'px-2 py-0.5 rounded-full text-xs font-medium border',
        colors[status] ?? 'bg-muted text-muted-foreground border-border'
      )}
    >
      {status}
    </span>
  )
}

export function FlashQueuePage() {
  const { data: flashData, isLoading } = useFlashJobs()
  const { data: usbData } = useUsbDevices()
  const { data: isoData } = useIsoImages()
  const cancelFlash = useCancelFlash()

  const usbMap = new Map(usbData?.devices.map((d) => [d.id, d]) ?? [])
  const isoMap = new Map(isoData?.images.map((i) => [i.id, i]) ?? [])

  const jobs = flashData?.jobs ?? []
  const activeJobs = jobs.filter(
    (j) => j.status === 'pending' || j.status === 'flashing'
  )
  const historyJobs = jobs.filter(
    (j) =>
      j.status === 'completed' ||
      j.status === 'failed' ||
      j.status === 'cancelled'
  )

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Flash Queue</h2>
          <p className="text-muted-foreground mt-1">
            Active and recent flash operations
          </p>
        </div>
      </div>

      <Card className="glass">
        <CardHeader>
          <CardTitle>
            Active Jobs
            {activeJobs.length > 0 && (
              <span className="ml-2 text-sm font-normal text-muted-foreground">
                ({activeJobs.length} running)
              </span>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
            </div>
          ) : activeJobs.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <Zap className="w-12 h-12 mx-auto mb-3 opacity-50" />
              <p>No active flash jobs</p>
              <p className="text-sm mt-1">
                Select a USB device and ISO from the dashboard to start flashing
              </p>
            </div>
          ) : (
            <div className="space-y-4">
              {activeJobs.map((job, i) => {
                const usb = usbMap.get(job.usb_id)
                const iso = isoMap.get(job.iso_id)
                return (
                  <motion.div
                    key={job.id}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: i * 0.05 }}
                    className="border rounded-lg p-4 space-y-3"
                  >
                    <div className="flex items-center justify-between">
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2">
                          <span className="font-medium text-sm truncate">
                            {iso?.filename ?? job.iso_id}
                          </span>
                          <StatusBadge status={job.status} />
                        </div>
                        <p className="text-xs text-muted-foreground font-mono mt-0.5">
                          {usb?.device_path ?? job.usb_id}
                          {usb?.model && ` \u2022 ${usb.model}`}
                        </p>
                      </div>
                      {job.status === 'flashing' && (
                        <button
                          onClick={() => cancelFlash.mutate(job.id)}
                          disabled={cancelFlash.isPending}
                          className="flex items-center gap-1 px-2 py-1 text-xs rounded-md text-red-500 hover:bg-red-500/10 transition-colors disabled:opacity-50"
                        >
                          <XCircle className="w-3 h-3" />
                          Cancel
                        </button>
                      )}
                    </div>

                    <Progress.Root
                      value={job.progress_percent}
                      className="ProgressRoot"
                    >
                      <Progress.Indicator
                        className="ProgressIndicator"
                        style={{ width: `${job.progress_percent}%` }}
                      />
                    </Progress.Root>

                    <div className="flex items-center justify-between text-xs text-muted-foreground">
                      <span>{job.progress_percent}%</span>
                      <div className="flex items-center gap-3">
                        {job.speed_bytes_per_sec > 0 && (
                          <span>{formatSpeed(job.speed_bytes_per_sec)}</span>
                        )}
                        {job.eta_seconds != null && (
                          <span>ETA: {formatEta(job.eta_seconds)}</span>
                        )}
                      </div>
                    </div>
                  </motion.div>
                )
              })}
            </div>
          )}
        </CardContent>
      </Card>

      <Card className="glass">
        <CardHeader>
          <CardTitle>History</CardTitle>
        </CardHeader>
        <CardContent>
          {historyJobs.length === 0 ? (
            <p className="text-muted-foreground text-sm">
              No completed or failed flash jobs yet
            </p>
          ) : (
            <div className="space-y-2">
              {historyJobs.map((job) => {
                const usb = usbMap.get(job.usb_id)
                const iso = isoMap.get(job.iso_id)
                return (
                  <div
                    key={job.id}
                    className="flex items-center justify-between py-2 border-b last:border-b-0"
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      {job.status === 'completed' ? (
                        <CheckCircle className="w-4 h-4 text-green-500 shrink-0" />
                      ) : (
                        <XCircle className="w-4 h-4 text-red-500 shrink-0" />
                      )}
                      <span className="text-sm truncate">
                        {iso?.filename ?? job.iso_id}
                      </span>
                      <span className="text-xs text-muted-foreground font-mono">
                        {usb?.device_path ?? job.usb_id}
                      </span>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      <StatusBadge status={job.status} />
                      {job.error_message && (
                        <span
                          className="text-xs text-red-500 max-w-[200px] truncate"
                          title={job.error_message}
                        >
                          {job.error_message}
                        </span>
                      )}
                    </div>
                  </div>
                )
              })}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
