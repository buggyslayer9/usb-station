import { Outlet } from '@tanstack/react-router'
import * as Toast from '@radix-ui/react-toast'
import { Sidebar } from './Sidebar'
import { TopBar } from './TopBar'

export function AppShell() {
  return (
    <Toast.Provider>
      <div className="flex h-screen bg-background">
        <Sidebar />
        <div className="flex-1 flex flex-col overflow-hidden">
          <TopBar />
          <main className="flex-1 overflow-y-auto p-6">
            <Outlet />
          </main>
        </div>
      </div>
      <Toast.Viewport className="fixed bottom-0 right-0 z-50 m-4 flex flex-col gap-2" />
    </Toast.Provider>
  )
}
