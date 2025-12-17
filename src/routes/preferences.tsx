import { createFileRoute } from '@tanstack/react-router'
import PreferencesForm from '@/components/PreferencesForm'

export const Route = createFileRoute('/preferences')({
  component: PreferencesPage,
})

function PreferencesPage() {
  return (
    <div className="min-h-screen p-8">
      <div className="max-w-md mx-auto space-y-6">
        <div>
          <h1 className="text-2xl font-bold">Preferences</h1>
          <p className="text-muted-foreground mt-1">
            Configure your OpenAI API key
          </p>
        </div>
        <PreferencesForm />
      </div>
    </div>
  )
}
