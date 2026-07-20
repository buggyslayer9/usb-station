import { motion } from 'framer-motion'
import { Disc3, Search } from 'lucide-react'
import { Card, CardHeader, CardTitle, CardContent } from '../ui/card'
import { useIsoImages } from '../../api/queries'
import { formatBytes } from '../../lib/utils'

export function IsoPage() {
  const { data, isLoading } = useIsoImages()

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">ISO Library</h2>
          <p className="text-muted-foreground mt-1">Available disk images</p>
        </div>
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="Search ISOs..."
            className="pl-9 pr-4 py-2 rounded-lg border bg-background text-sm w-64"
          />
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {data?.images.map((iso, i) => (
          <motion.div
            key={iso.id}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.03 }}
          >
            <Card className="glass group cursor-pointer hover:border-primary/50 transition-colors">
              <CardHeader>
                <div className="flex items-center gap-3">
                  <Disc3 className="w-8 h-8 text-primary" />
                  <div className="min-w-0">
                    <CardTitle className="text-sm truncate">{iso.filename}</CardTitle>
                    {iso.detected_os && (
                      <p className="text-xs text-muted-foreground">
                        {iso.detected_os} {iso.detected_version && `• ${iso.detected_version}`}
                      </p>
                    )}
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">{formatBytes(iso.file_size_bytes)}</span>
                  {iso.category && <span className="text-xs px-2 py-0.5 rounded bg-primary/10">{iso.category}</span>}
                </div>
              </CardContent>
            </Card>
          </motion.div>
        ))}
        {data?.images.length === 0 && !isLoading && (
          <div className="col-span-full text-center py-12 text-muted-foreground">
            <Disc3 className="w-12 h-12 mx-auto mb-3 opacity-50" />
            <p>No ISO images found</p>
            <p className="text-sm mt-1">Place ISO files in /storage/iso or upload via the file browser</p>
          </div>
        )}
      </div>
    </div>
  )
}
