/** Logo de Numéris : bobines de cassette sur fond d'accent. */
export function Logo({ size }: { size: number }) {
  return (
    <span className="logo" style={{ width: size, height: size }} aria-hidden="true">
      <svg
        width={size * 0.66}
        height={size * 0.66}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2.2"
        strokeLinecap="round"
      >
        <circle cx="8.5" cy="12" r="2.5" />
        <circle cx="15.5" cy="12" r="2.5" />
        <path d="M8.5 14.5h7" />
      </svg>
    </span>
  );
}
