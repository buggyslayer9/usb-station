import { Card, CardHeader, CardTitle, CardContent } from '../ui/card'

export function SettingsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold tracking-tight">Settings</h2>
        <p className="text-muted-foreground mt-1">Configure USB Station</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card className="glass">
          <CardHeader>
            <CardTitle>Flash Defaults</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm">Verify after flash</span>
              <input type="checkbox" defaultChecked className="toggle" />
            </div>
            <div>
              <label className="text-sm">Max concurrent flashes</label>
              <input type="number" defaultValue={2} className="w-full mt-1 px-3 py-2 rounded-lg border bg-background text-sm" />
            </div>
          </CardContent>
        </Card>

        <Card className="glass">
          <CardHeader>
            <CardTitle>Storage</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <label className="text-sm">ISO directory</label>
              <input type="text" defaultValue="/storage/iso" className="w-full mt-1 px-3 py-2 rounded-lg border bg-background text-sm font-mono" />
            </div>
            <div>
              <label className="text-sm">Downloads directory</label>
              <input type="text" defaultValue="/storage/downloads" className="w-full mt-1 px-3 py-2 rounded-lg border bg-background text-sm font-mono" />
            </div>
          </CardContent>
        </Card>

        <Card className="glass">
          <CardHeader>
            <CardTitle>Theme</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-between">
              <span className="text-sm">Dark mode</span>
              <input type="checkbox" defaultChecked className="toggle" />
            </div>
          </CardContent>
        </Card>

        <Card className="glass">
          <CardHeader>
            <CardTitle>Notifications</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm">Browser notifications</span>
              <input type="checkbox" defaultChecked className="toggle" />
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
