import type { Finding } from "./types";

type IconProps = {
  className?: string;
};

export function CompassEmblem({ className }: IconProps) {
  return (
    <svg
      aria-hidden="true"
      className={className}
      viewBox="0 0 32 32"
      fill="none"
    >
      <circle className="emblem-ring" cx="16" cy="16" r="11.25" />
      <path className="emblem-ticks" d="M16 2.75v3M16 26.25v3M2.75 16h3M26.25 16h3" />
      <path className="emblem-needle-back" d="m10.25 22 3.8-8.05L22 10.2l-3.72 7.88L10.25 22Z" />
      <path className="emblem-needle" d="m14.05 13.95 7.95-3.75-3.72 7.88-4.23-4.13Z" />
      <circle className="emblem-pivot" cx="16.15" cy="16.05" r="1.75" />
      <circle className="emblem-waypoint" cx="23.9" cy="8.25" r="1.55" />
    </svg>
  );
}

export function FolderIcon({ className }: IconProps) {
  return (
    <svg aria-hidden="true" className={className} viewBox="0 0 24 24" fill="none">
      <path d="M3.5 6.5h6l2 2h9v9.75A1.75 1.75 0 0 1 18.75 20H5.25a1.75 1.75 0 0 1-1.75-1.75V6.5Z" />
      <path d="M3.5 9h17" />
    </svg>
  );
}

export function MonitorIcon({ className }: IconProps) {
  return (
    <svg aria-hidden="true" className={className} viewBox="0 0 24 24" fill="none">
      <rect x="3" y="4" width="18" height="13" rx="2" />
      <path d="M8 21h8M12 17v4" />
    </svg>
  );
}

export function ThemeIcon({ className }: IconProps) {
  return (
    <svg aria-hidden="true" className={className} viewBox="0 0 20 20" fill="none">
      <circle cx="10" cy="10" r="4.25" />
      <path d="M10 1.75v2M10 16.25v2M1.75 10h2M16.25 10h2M4.16 4.16l1.4 1.4M14.44 14.44l1.4 1.4M15.84 4.16l-1.4 1.4M5.56 14.44l-1.4 1.4" />
    </svg>
  );
}

export function LockIcon({ className }: IconProps) {
  return (
    <svg aria-hidden="true" className={className} viewBox="0 0 16 16" fill="none">
      <rect x="3.25" y="6.75" width="9.5" height="6.5" rx="1.5" />
      <path d="M5.35 6.75V5.1a2.65 2.65 0 0 1 5.3 0v1.65" />
    </svg>
  );
}

export function ArrowIcon({ className }: IconProps) {
  return (
    <svg aria-hidden="true" className={className} viewBox="0 0 20 20" fill="none">
      <path d="M3.5 10h12M11.5 6l4 4-4 4" />
    </svg>
  );
}

export function EvidenceIcon({ className }: IconProps) {
  return (
    <svg aria-hidden="true" className={className} viewBox="0 0 20 20" fill="none">
      <path d="M4 3.25h8.5L16 6.75v10H4v-13.5Z" />
      <path d="M12.5 3.25v3.5H16M6.75 10h6.5M6.75 13h4.5" />
    </svg>
  );
}

export function RouteIcon({ className }: IconProps) {
  return (
    <svg aria-hidden="true" className={className} viewBox="0 0 20 20" fill="none">
      <circle cx="4" cy="15.5" r="1.75" />
      <circle cx="15.5" cy="4.5" r="1.75" />
      <path d="M5.75 15.5h2.1c1.25 0 2.25-1 2.25-2.25v-6.5c0-1.25 1-2.25 2.25-2.25h1.4" />
    </svg>
  );
}

export function ShareIcon({ className }: IconProps) {
  return (
    <svg aria-hidden="true" className={className} viewBox="0 0 20 20" fill="none">
      <circle cx="5" cy="10" r="2" />
      <circle cx="15" cy="4.5" r="2" />
      <circle cx="15" cy="15.5" r="2" />
      <path d="m6.75 9 6.5-3.5M6.75 11l6.5 3.5" />
    </svg>
  );
}

export function TechnicalIcon({ className }: IconProps) {
  return (
    <svg aria-hidden="true" className={className} viewBox="0 0 20 20" fill="none">
      <path d="M3 5.25h14M3 10h14M3 14.75h14" />
      <circle cx="6.25" cy="5.25" r="1.5" />
      <circle cx="13.5" cy="10" r="1.5" />
      <circle cx="8.5" cy="14.75" r="1.5" />
    </svg>
  );
}

export function SeverityIcon({
  severity,
  className,
}: IconProps & { severity: Finding["severity"] }) {
  return (
    <svg
      aria-hidden="true"
      className={className}
      viewBox="0 0 24 24"
      fill="none"
    >
      {severity === "problem" ? (
        <>
          <path d="M12 3.5 20.5 12 12 20.5 3.5 12 12 3.5Z" />
          <path d="M12 7.75v5.5M12 16.25v.15" />
        </>
      ) : null}
      {severity === "warning" ? (
        <>
          <path d="M12 3.5 21 19H3L12 3.5Z" />
          <path d="M12 8.5v5M12 16.25v.15" />
        </>
      ) : null}
      {severity === "suggestion" ? (
        <>
          <circle cx="12" cy="12" r="8.5" />
          <path d="m8.5 15.5 2.2-5 4.8-2-2.15 4.85-4.85 2.15Z" />
        </>
      ) : null}
      {severity === "info" ? (
        <>
          <circle cx="12" cy="12" r="8.5" />
          <path d="M12 10.5v6M12 7.5v.15" />
        </>
      ) : null}
    </svg>
  );
}

export function CalibrationGraphic() {
  return (
    <svg className="calibration-graphic" viewBox="0 0 280 220" fill="none" aria-hidden="true">
      <circle className="graphic-orbit orbit-one" cx="140" cy="110" r="74" />
      <circle className="graphic-orbit orbit-two" cx="140" cy="110" r="47" />
      <path className="graphic-crosshair" d="M140 17v26M140 177v26M47 110h26M207 110h26" />
      <path className="graphic-route" d="M76 165c20-11 26-30 33-49 8-22 19-39 46-45 18-4 34 0 50 9" />
      <circle className="graphic-waypoint" cx="76" cy="165" r="5" />
      <circle className="graphic-waypoint" cx="205" cy="80" r="5" />
      <path className="graphic-needle-back" d="m108 148 19-47 45-22-20 46-44 23Z" />
      <path className="graphic-needle" d="m127 101 45-22-20 46-25-24Z" />
      <circle className="graphic-pivot" cx="140" cy="113" r="8" />
      <text x="37" y="42">N 31°14′</text>
      <text x="184" y="195">E 121°28′</text>
    </svg>
  );
}
