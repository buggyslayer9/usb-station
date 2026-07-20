import { useState } from 'react'
import { Layers, Play, Loader2 } from 'lucide-react'
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  CardFooter,
} from '../ui/card'
import {
  useUsbDevices,
  useIsoImages,
  useCreateBatch,
  type FlashBatch,
} from '../../api/queries'
import { api } from '../../api/client'
import { useQuery } from '@tanstack/react-query'

type BatchMode = 'Clone' | 'Sequential' | 'SmartAssign' | 'ManualQueue'

const modeDescriptions: Record<BatchMode, string> = {
  Clone: 'Flash one ISO to multiple USBs simultaneously',
  Sequential: 'Pair N ISOs with N USBs in order',
  SmartAssign: 'Auto-match ISOs to USBs by capacity',
  ManualQueue: 'Manually add individual flash jobs to the batch',
}

export function BatchPage() {
  const [name, setName] = useState('')
  const [mode, setMode] = useState<BatchMode>('Clone')
  const [selectedIsos, setSelectedIsos] = useState<string[]>([])
  const [selectedUsbs, setSelectedUsbs] = useState<string[]>([])
  const [maxConcurrent, setMaxConcurrent] = useState(2)

  const { data: usbData } = useUsbDevices()
  const { data: isoData } = useIsoImages()
  const createBatch = useCreateBatch()
  const { data: batchData } = useQuery({
    queryKey: ['batches'],
    queryFn: () => api.get<{ batches: FlashBatch[] }>('/batch'),
  })

  const handleCreateBatch = () => {
    createBatch.mutate({
      name: name || `${mode} Batch - ${new Date().toLocaleString()}`,
      mode,
      iso_ids: selectedIsos,
      usb_ids: selectedUsbs,
      max_concurrent: maxConcurrent,
    })
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold tracking-tight">Batch Flash</h2>
        <p className="text-muted-foreground mt-1">
          Flash multiple USBs at once
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-4">
          <Card className="glass">
            <CardHeader>
              <CardTitle>New Batch</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <label className="text-sm font-medium">Batch Name</label>
                <input
                  type="text"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="e.g., Ubuntu 24.04 Workshop USBs"
                  className="w-full mt-1 px-3 py-2 rounded-lg border bg-background text-sm"
                />
              </div>

              <div>
                <label className="text-sm font-medium">Flash Mode</label>
                <div className="grid grid-cols-2 gap-2 mt-1">
                  {(Object.keys(modeDescriptions) as BatchMode[]).map((m) => (
                    <button
                      key={m}
                      onClick={() => setMode(m)}
                      className={`text-left p-3 rounded-lg border text-sm transition-colors ${
                        mode === m
                          ? 'border-primary bg-primary/5'
                          : 'hover:border-muted-foreground/30'
                      }`}
                    >
                      <div className="font-medium">{m}</div>
                      <div className="text-xs text-muted-foreground mt-1">
                        {modeDescriptions[m]}
                      </div>
                    </button>
                  ))}
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-sm font-medium">
                    ISO Images ({selectedIsos.length})
                  </label>
                  <div className="mt-1 border rounded-lg max-h-40 overflow-y-auto">
                    {isoData?.images.map((iso) => (
                      <label
                        key={iso.id}
                        className="flex items-center gap-2 px-3 py-2 text-sm hover:bg-muted/50 cursor-pointer"
                      >
                        <input
                          type="checkbox"
                          checked={selectedIsos.includes(iso.id)}
                          onChange={(e) => {
                            if (e.target.checked) {
                              if (mode === 'Clone') {
                                setSelectedIsos([iso.id])
                              } else {
                                setSelectedIsos([...selectedIsos, iso.id])
                              }
                            } else {
                              setSelectedIsos(
                                selectedIsos.filter((id) => id !== iso.id)
                              )
                            }
                          }}
                          className="rounded"
                        />
                        <span className="truncate">{iso.filename}</span>
                      </label>
                    ))}
                  </div>
                </div>
                <div>
                  <label className="text-sm font-medium">
                    USB Targets ({selectedUsbs.length})
                  </label>
                  <div className="mt-1 border rounded-lg max-h-40 overflow-y-auto">
                    {usbData?.devices
                      .filter((d) => !d.is_system_disk)
                      .map((usb) => (
                        <label
                          key={usb.id}
                          className="flex items-center gap-2 px-3 py-2 text-sm hover:bg-muted/50 cursor-pointer"
                        >
                          <input
                            type="checkbox"
                            checked={selectedUsbs.includes(usb.id)}
                            onChange={(e) => {
                              if (e.target.checked) {
                                setSelectedUsbs([...selectedUsbs, usb.id])
                              } else {
                                setSelectedUsbs(
                                  selectedUsbs.filter((id) => id !== usb.id)
                                )
                              }
                            }}
                            className="rounded"
                          />
                          <span className="font-mono text-xs">
                            {usb.device_path}
                          </span>
                          <span className="text-muted-foreground ml-auto text-xs">
                            {usb.model || usb.vendor || 'USB'}
                          </span>
                        </label>
                      ))}
                  </div>
                </div>
              </div>

              <div>
                <label className="text-sm font-medium">
                  Max Concurrent Flashes: {maxConcurrent}
                </label>
                <input
                  type="range"
                  min={1}
                  max={8}
                  value={maxConcurrent}
                  onChange={(e) => setMaxConcurrent(Number(e.target.value))}
                  className="w-full mt-1"
                />
              </div>

              {createBatch.error && (
                <div className="p-3 rounded-lg bg-red-500/10 text-red-500 text-sm">
                  {createBatch.error instanceof Error
                    ? createBatch.error.message
                    : 'Failed to create batch'}
                </div>
              )}

              {createBatch.isSuccess && (
                <div className="p-3 rounded-lg bg-green-500/10 text-green-500 text-sm">
                  Batch created successfully
                </div>
              )}
            </CardContent>
            <CardFooter>
              <button
                onClick={handleCreateBatch}
                disabled={
                  selectedIsos.length === 0 ||
                  selectedUsbs.length === 0 ||
                  createBatch.isPending
                }
                className="flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 disabled:opacity-50"
              >
                {createBatch.isPending ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <Play className="w-4 h-4" />
                )}
                Create & Start Batch
              </button>
            </CardFooter>
          </Card>
        </div>

        <div className="space-y-4">
          <h3 className="text-lg font-semibold">Active Batches</h3>
          {!batchData || batchData.batches.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <Layers className="w-12 h-12 mx-auto mb-3 opacity-50" />
              <p className="text-sm">No active batches</p>
              <p className="text-xs mt-1">
                Configure and start a batch on the left
              </p>
            </div>
          ) : (
            <div className="space-y-2">
              {batchData.batches.map((batch) => (
                <Card key={batch.id} className="glass">
                  <CardContent className="p-4">
                    <div className="flex justify-between items-center">
                      <div>
                        <p className="text-sm font-medium">{batch.name}</p>
                        <p className="text-xs text-muted-foreground">
                          {batch.mode}
                        </p>
                      </div>
                      <span className="text-xs px-2 py-0.5 rounded-full bg-primary/10">
                        {batch.status}
                      </span>
                    </div>
                    <div className="mt-2 flex gap-4 text-xs text-muted-foreground">
                      <span>
                        {batch.completed_jobs}/{batch.total_jobs} done
                      </span>
                      {batch.failed_jobs > 0 && (
                        <span className="text-red-500">
                          {batch.failed_jobs} failed
                        </span>
                      )}
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
