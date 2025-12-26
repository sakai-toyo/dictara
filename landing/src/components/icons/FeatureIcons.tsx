import type { SVGProps } from "react";

type IconProps = SVGProps<SVGSVGElement>;

// Lightning Fast - Stylized bolt with motion lines
export function IconLightning(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {...props}>
      {/* Motion lines */}
      <path d="M3 8h3M3 12h2M3 16h3" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" opacity="0.6" />
      {/* Lightning bolt */}
      <path
        d="M13 2L8 12h4l-1 10 7-12h-5l2-8z"
        fill="currentColor"
        stroke="currentColor"
        strokeWidth="1"
        strokeLinejoin="round"
      />
    </svg>
  );
}

// 90+ Languages - Globe with speech elements
export function IconLanguages(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {...props}>
      {/* Globe */}
      <circle cx="12" cy="12" r="9" stroke="currentColor" strokeWidth="1.5" />
      <ellipse cx="12" cy="12" rx="4" ry="9" stroke="currentColor" strokeWidth="1.5" />
      <path d="M3 12h18" stroke="currentColor" strokeWidth="1.5" />
      <path d="M4.5 7h15M4.5 17h15" stroke="currentColor" strokeWidth="1.5" opacity="0.6" />
      {/* Speech indicator */}
      <circle cx="19" cy="5" r="3.5" fill="currentColor" />
      <text x="19" y="6.5" textAnchor="middle" fontSize="5" fill="var(--background, #0f0a1a)" fontWeight="bold">A</text>
    </svg>
  );
}

// Privacy First - Shield with keyhole
export function IconPrivacy(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {...props}>
      {/* Shield */}
      <path
        d="M12 2L4 6v5c0 5.25 3.4 10.15 8 11.5 4.6-1.35 8-6.25 8-11.5V6l-8-4z"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinejoin="round"
      />
      {/* Keyhole */}
      <circle cx="12" cy="10" r="2" fill="currentColor" />
      <path d="M12 12v4" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" />
    </svg>
  );
}

// Powered by Whisper - Sound wave with AI sparkle
export function IconWhisper(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {...props}>
      {/* Sound waves */}
      <path d="M4 12h2" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
      <path d="M7 8v8" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
      <path d="M10 5v14" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
      <path d="M13 7v10" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
      <path d="M16 9v6" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
      {/* AI sparkle */}
      <path d="M20 4l.5 1.5L22 6l-1.5.5L20 8l-.5-1.5L18 6l1.5-.5L20 4z" fill="currentColor" />
      <path d="M19 14l.3 1 1 .3-1 .3-.3 1-.3-1-1-.3 1-.3.3-1z" fill="currentColor" opacity="0.7" />
    </svg>
  );
}

// Simple Workflow - FN key with tap indicator
export function IconWorkflow(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {...props}>
      {/* Key cap */}
      <rect x="4" y="8" width="16" height="12" rx="2" stroke="currentColor" strokeWidth="1.5" />
      <rect x="6" y="10" width="12" height="8" rx="1" stroke="currentColor" strokeWidth="1" opacity="0.5" />
      {/* FN text */}
      <text x="12" y="16" textAnchor="middle" fontSize="5" fill="currentColor" fontWeight="bold" fontFamily="system-ui">FN</text>
      {/* Tap indicator */}
      <path d="M12 2v3" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      <path d="M9 4l3-2 3 2" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

// Auto Paste - Cursor with text appearing
export function IconAutoPaste(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {...props}>
      {/* Text cursor */}
      <path d="M12 4v16" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
      <path d="M9 4h6M9 20h6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      {/* Text lines appearing */}
      <path d="M16 8h5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" opacity="0.4" />
      <path d="M16 12h6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" opacity="0.7" />
      <path d="M16 16h4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      {/* Sparkle effect */}
      <circle cx="19" cy="5" r="1" fill="currentColor" />
    </svg>
  );
}
