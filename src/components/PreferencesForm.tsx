import { useForm } from '@tanstack/react-form'
import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'
import { Button } from './ui/button'
import { Input } from './ui/input'
import { Label } from './ui/label'

function maskApiKey(key: string): string {
  if (key.length <= 12) return key
  return `${key.slice(0, 8)}...${key.slice(-4)}`
}

export default function PreferencesForm() {
  const [existingKey, setExistingKey] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [isTesting, setIsTesting] = useState(false)
  const [testResult, setTestResult] = useState<'success' | 'error' | null>(null)
  const [saveSuccess, setSaveSuccess] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  // Load existing API key on mount
  useEffect(() => {
    async function loadKey() {
      try {
        const key = await invoke<string | null>('load_openai_key')
        setExistingKey(key)
        console.log('[PreferencesForm] Loaded API key from keychain:', key ? 'found' : 'not found')
      } catch (e) {
        console.error('[PreferencesForm] Failed to load API key:', e)
      } finally {
        setIsLoading(false)
      }
    }
    loadKey()
  }, [])

  const form = useForm({
    defaultValues: {
      apiKey: '',
    },
    onSubmit: async ({ value }) => {
      console.log('[PreferencesForm] Saving API key...')
      setErrorMessage(null)
      setSaveSuccess(false)

      try {
        await invoke('save_openai_key', { key: value.apiKey })
        console.log('[PreferencesForm] API key saved successfully')
        setExistingKey(value.apiKey)
        setSaveSuccess(true)
        form.reset()
        setTestResult(null)
      } catch (e) {
        console.error('[PreferencesForm] Failed to save API key:', e)
        setErrorMessage(`Failed to save: ${e}`)
      }
    },
  })

  const handleTest = async () => {
    const apiKey = form.getFieldValue('apiKey')
    if (!apiKey) return

    console.log('[PreferencesForm] Testing API key...')
    setIsTesting(true)
    setTestResult(null)
    setErrorMessage(null)

    try {
      const isValid = await invoke<boolean>('test_openai_key', { key: apiKey })
      console.log('[PreferencesForm] API key test result:', isValid ? 'valid' : 'invalid')
      setTestResult(isValid ? 'success' : 'error')
      if (!isValid) {
        setErrorMessage('Invalid API key')
      }
    } catch (e) {
      console.error('[PreferencesForm] Failed to test API key:', e)
      setTestResult('error')
      setErrorMessage(`Test failed: ${e}`)
    } finally {
      setIsTesting(false)
    }
  }

  const handleDelete = async () => {
    console.log('[PreferencesForm] Deleting API key...')
    try {
      await invoke('delete_openai_key')
      console.log('[PreferencesForm] API key deleted successfully')
      setExistingKey(null)
      setTestResult(null)
      setSaveSuccess(false)
    } catch (e) {
      console.error('[PreferencesForm] Failed to delete API key:', e)
      setErrorMessage(`Failed to delete: ${e}`)
    }
  }

  if (isLoading) {
    return <div className="text-muted-foreground">Loading...</div>
  }

  return (
    <div className="space-y-6">
      {/* Current key status */}
      {existingKey && (
        <div className="p-4 bg-muted rounded-lg">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-muted-foreground">Current API Key</p>
              <p className="font-mono text-sm">{maskApiKey(existingKey)}</p>
            </div>
            <Button
              variant="destructive"
              size="sm"
              onClick={handleDelete}
            >
              Delete
            </Button>
          </div>
        </div>
      )}

      {/* Form */}
      <form
        onSubmit={(e) => {
          e.preventDefault()
          e.stopPropagation()
          form.handleSubmit()
        }}
        className="space-y-4"
      >
        <div className="space-y-2">
          <Label htmlFor="apiKey">
            {existingKey ? 'Update API Key' : 'OpenAI API Key'}
          </Label>
          <form.Field
            name="apiKey"
            validators={{
              onChange: ({ value }) => {
                if (!value) return 'API key is required'
                if (value.length < 20) return 'API key is too short'
                if (!value.startsWith('sk-')) return 'API key should start with sk-'
                return undefined
              },
            }}
          >
            {(field) => (
              <div className="space-y-1">
                <Input
                  id="apiKey"
                  type="password"
                  placeholder="sk-..."
                  value={field.state.value}
                  onChange={(e) => {
                    field.handleChange(e.target.value)
                    setTestResult(null)
                    setSaveSuccess(false)
                  }}
                  onBlur={field.handleBlur}
                />
                {field.state.meta.isTouched && field.state.meta.errors.length > 0 && (
                  <p className="text-sm text-destructive">
                    {field.state.meta.errors.join(', ')}
                  </p>
                )}
              </div>
            )}
          </form.Field>
        </div>

        {/* Feedback messages */}
        {errorMessage && (
          <p className="text-sm text-destructive">{errorMessage}</p>
        )}
        {testResult === 'success' && (
          <p className="text-sm text-green-600">API key is valid!</p>
        )}
        {saveSuccess && (
          <p className="text-sm text-green-600">API key saved successfully!</p>
        )}

        {/* Action buttons */}
        <div className="flex gap-2">
          <form.Subscribe
            selector={(state) => ({
              canSubmit: state.canSubmit,
              isSubmitting: state.isSubmitting,
              apiKey: state.values.apiKey,
            })}
          >
            {({ canSubmit, isSubmitting, apiKey }) => (
              <>
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleTest}
                  disabled={!apiKey || isTesting || !canSubmit}
                >
                  {isTesting ? 'Testing...' : 'Test Key'}
                </Button>
                <Button
                  type="submit"
                  disabled={!canSubmit || isSubmitting || testResult !== 'success'}
                >
                  {isSubmitting ? 'Saving...' : 'Save'}
                </Button>
              </>
            )}
          </form.Subscribe>
        </div>
      </form>

      <p className="text-xs text-muted-foreground">
        Your API key is stored securely in the macOS Keychain.
      </p>
    </div>
  )
}
