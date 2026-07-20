import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from './client'

export type UsbDevice = {
  id: string
  device_path: string
  vendor: string | null
  model: string | null
  serial: string | null
  capacity_bytes: number
  is_mounted: boolean
  is_system_disk: boolean
  health: string
}

export type IsoImage = {
  id: string
  filename: string
  file_size_bytes: number
  detected_os: string | null
  detected_version: string | null
  category: string | null
}

export type FlashJob = {
  id: string
  iso_id: string
  usb_id: string
  batch_id: string | null
  status: string
  progress_percent: number
  speed_bytes_per_sec: number
  eta_seconds: number | null
  error_message: string | null
}

export type FlashBatch = {
  id: string
  name: string
  mode: string
  status: string
  total_jobs: number
  completed_jobs: number
  failed_jobs: number
}

export type BatchCreateRequest = {
  name: string
  mode: 'Clone' | 'Sequential' | 'SmartAssign' | 'ManualQueue'
  iso_ids: string[]
  usb_ids: string[]
  max_concurrent?: number
}

export const useUsbDevices = () =>
  useQuery({
    queryKey: ['usb-devices'],
    queryFn: () => api.get<{ devices: UsbDevice[] }>('/usb'),
    refetchInterval: 5000,
  })

export const useIsoImages = () =>
  useQuery({
    queryKey: ['iso-images'],
    queryFn: () => api.get<{ images: IsoImage[] }>('/iso'),
  })

export const useFlashJobs = () =>
  useQuery({
    queryKey: ['flash-jobs'],
    queryFn: () => api.get<{ jobs: FlashJob[] }>('/flash'),
  })

export const useFlashJobStatus = (id: string) =>
  useQuery({
    queryKey: ['flash-job', id],
    queryFn: () => api.get<FlashJob>(`/flash/${id}`),
    refetchInterval: 1000,
  })

export const useStartFlash = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (data: { iso_id: string; usb_id: string }) =>
      api.post('/flash', data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['flash-jobs'] }),
  })
}

export const useCancelFlash = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => api.post(`/flash/${id}/cancel`),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['flash-jobs'] }),
  })
}

export const useCreateBatch = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (data: BatchCreateRequest) => api.post('/batch', data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['batches'] }),
  })
}

export const useBatchProgress = (id: string) =>
  useQuery({
    queryKey: ['batch', id],
    queryFn: () => api.get<FlashBatch>(`/batch/${id}`),
    refetchInterval: 2000,
    enabled: !!id,
  })
