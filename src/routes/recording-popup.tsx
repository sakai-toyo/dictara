import { createFileRoute } from '@tanstack/react-router'
import RecordingPopup from '../RecordingPopup'

export const Route = createFileRoute('/recording-popup')({
  component: RecordingPopup,
})
