import { error as logError } from '@tauri-apps/plugin-log'
import { RotateCcw } from 'lucide-react'
import { Switch } from '../ui/switch'
import { Label } from '../ui/label'
import { Button } from '../ui/button'
import { Input } from '../ui/input'
import { Textarea } from '../ui/textarea'
import { useIsAutostartEnabled, useToggleAutostart } from '@/hooks/useAutostart'
import { useRestartOnboarding } from '@/hooks/useOnboardingNavigation'
import { useAppConfig } from '@/hooks/useAppConfig'
import { useSaveAppConfig } from '@/hooks/useSaveAppConfig'
import { useEffect, useMemo, useState } from 'react'

const MIN_ALLOWED_SPEECH_DURATION_MS = 100
const MAX_ALLOWED_SPEECH_DURATION_MS = 10_000

export function System() {
  const { data: isEnabled, isLoading: isCheckingStatus } = useIsAutostartEnabled()
  const { toggle, isLoading: isToggling } = useToggleAutostart()
  const restartOnboarding = useRestartOnboarding()
  const { data: appConfig, isLoading: isAppConfigLoading } = useAppConfig()
  const saveAppConfig = useSaveAppConfig()
  const [postProcessEnabled, setPostProcessEnabled] = useState(true)
  const [postProcessModel, setPostProcessModel] = useState('')
  const [postProcessPrompt, setPostProcessPrompt] = useState('')
  const [minSpeechDurationMs, setMinSpeechDurationMs] = useState('500')

  useEffect(() => {
    if (!appConfig) return
    setPostProcessEnabled(appConfig.postProcessEnabled ?? true)
    setPostProcessModel(appConfig.postProcessModel ?? '')
    setPostProcessPrompt(appConfig.postProcessPrompt ?? '')
    setMinSpeechDurationMs(String(appConfig.minSpeechDurationMs ?? 500))
  }, [appConfig])

  const handleToggleAutostart = async (checked: boolean) => {
    try {
      await toggle(!checked) // Toggle to the opposite of current state
    } catch (e) {
      logError(`[System] Failed to toggle autostart: ${e}`)
    }
  }

  const isPostProcessDirty = useMemo(() => {
    if (!appConfig) return false
    return (
      postProcessEnabled !== (appConfig.postProcessEnabled ?? true) ||
      postProcessModel !== (appConfig.postProcessModel ?? '') ||
      postProcessPrompt !== (appConfig.postProcessPrompt ?? '')
    )
  }, [appConfig, postProcessEnabled, postProcessModel, postProcessPrompt])

  const minSpeechDurationNumber = Number(minSpeechDurationMs)
  const isMinSpeechDurationInvalid =
    !Number.isInteger(minSpeechDurationNumber) ||
    minSpeechDurationNumber < MIN_ALLOWED_SPEECH_DURATION_MS ||
    minSpeechDurationNumber > MAX_ALLOWED_SPEECH_DURATION_MS

  const isRecordingSettingsDirty = useMemo(() => {
    if (!appConfig) return false
    return minSpeechDurationNumber !== (appConfig.minSpeechDurationMs ?? 500)
  }, [appConfig, minSpeechDurationNumber])

  const handleSavePostProcessSettings = async () => {
    try {
      await saveAppConfig.mutateAsync({
        postProcessEnabled,
        postProcessModel,
        postProcessPrompt,
      })
    } catch (e) {
      logError(`[System] Failed to save post-process settings: ${e}`)
    }
  }

  const handleSaveRecordingSettings = async () => {
    if (isMinSpeechDurationInvalid) return
    try {
      await saveAppConfig.mutateAsync({
        minSpeechDurationMs: minSpeechDurationNumber,
      })
    } catch (e) {
      logError(`[System] Failed to save recording settings: ${e}`)
    }
  }

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <p className="text-sm text-muted-foreground">System Settings</p>
        <p className="text-sm">Configure how Dictara integrates with your system.</p>
      </div>

      <div className="flex items-center justify-between rounded-lg border p-4">
        <div className="space-y-0.5">
          <Label htmlFor="autostart" className="text-base">
            Launch at Startup
          </Label>
          <p className="text-sm text-muted-foreground">
            Automatically start Dictara when you log in to your computer
          </p>
        </div>
        <Switch
          id="autostart"
          checked={isEnabled ?? false}
          onCheckedChange={handleToggleAutostart}
          disabled={isCheckingStatus || isToggling}
        />
      </div>

      <div className="flex items-center justify-between rounded-lg border p-4">
        <div className="space-y-0.5">
          <Label className="text-base">Restart Onboarding</Label>
          <p className="text-sm text-muted-foreground">
            Go through the initial setup wizard again. This can help troubleshoot configuration
            issues or reset your preferences
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => restartOnboarding.mutate()}
          disabled={restartOnboarding.isPending}
        >
          <RotateCcw className="mr-2 h-4 w-4" />
          {restartOnboarding.isPending ? 'Restarting...' : 'Restart'}
        </Button>
      </div>

      <div className="space-y-3 rounded-lg border p-4">
        <div className="space-y-2">
          <Label htmlFor="min-speech-duration-ms" className="text-base">
            Minimum Speech Duration (ms)
          </Label>
          <p className="text-sm text-muted-foreground">
            Skip transcription when detected speech is shorter than this value
          </p>
          <Input
            id="min-speech-duration-ms"
            type="number"
            min={MIN_ALLOWED_SPEECH_DURATION_MS}
            max={MAX_ALLOWED_SPEECH_DURATION_MS}
            step={100}
            value={minSpeechDurationMs}
            onChange={(e) => setMinSpeechDurationMs(e.target.value)}
            disabled={isAppConfigLoading || saveAppConfig.isPending}
          />
          {isMinSpeechDurationInvalid ? (
            <p className="text-sm text-red-600">
              Enter a value between {MIN_ALLOWED_SPEECH_DURATION_MS} and{' '}
              {MAX_ALLOWED_SPEECH_DURATION_MS} ms.
            </p>
          ) : null}
        </div>

        <div className="flex justify-end">
          <Button
            onClick={handleSaveRecordingSettings}
            disabled={
              isAppConfigLoading ||
              saveAppConfig.isPending ||
              !isRecordingSettingsDirty ||
              isMinSpeechDurationInvalid
            }
          >
            {saveAppConfig.isPending ? 'Saving...' : 'Save Recording Settings'}
          </Button>
        </div>
      </div>

      <div className="space-y-3 rounded-lg border p-4">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label htmlFor="post-process-enabled" className="text-base">
              Transcription Post-Processing
            </Label>
            <p className="text-sm text-muted-foreground">
              Automatically fix transcription typos and missing characters before pasting
            </p>
          </div>
          <Switch
            id="post-process-enabled"
            checked={postProcessEnabled}
            onCheckedChange={setPostProcessEnabled}
            disabled={isAppConfigLoading || saveAppConfig.isPending}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="post-process-model">Model Name</Label>
          <Input
            id="post-process-model"
            value={postProcessModel}
            onChange={(e) => setPostProcessModel(e.target.value)}
            disabled={isAppConfigLoading || saveAppConfig.isPending}
            placeholder="gpt-4.1-nano"
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="post-process-prompt">Post-Process Prompt</Label>
          <Textarea
            id="post-process-prompt"
            value={postProcessPrompt}
            onChange={(e) => setPostProcessPrompt(e.target.value)}
            disabled={isAppConfigLoading || saveAppConfig.isPending}
            rows={8}
          />
        </div>

        <div className="flex justify-end">
          <Button
            onClick={handleSavePostProcessSettings}
            disabled={isAppConfigLoading || saveAppConfig.isPending || !isPostProcessDirty}
          >
            {saveAppConfig.isPending ? 'Saving...' : 'Save Post-Process Settings'}
          </Button>
        </div>
      </div>
    </div>
  )
}
