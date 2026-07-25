import type { MascotState } from "../types";

interface Props {
  state: MascotState;
  onClick?: () => void;
}

const INK = "#4A3524";

function Face({ state }: { state: MascotState }) {
  const blush = (
    <>
      <ellipse cx="-20" cy="8" rx="5" ry="3" fill="#F3A6A0" opacity={0.55} />
      <ellipse cx="20" cy="8" rx="5" ry="3" fill="#F3A6A0" opacity={0.55} />
    </>
  );
  switch (state) {
    case "happy":
      return (
        <g transform="translate(100,74)">
          {blush}
          <circle cx="12" cy="-2" r="3.4" fill={INK} />
          <circle cx="13.1" cy="-3.2" r="1" fill="#fff" />
          <path d="M-18,-4 Q-12,-9 -6,-4" stroke={INK} strokeWidth={2.5} fill="none" strokeLinecap="round" />
          <path d="M-13,9 Q0,19 13,9" stroke={INK} strokeWidth={2.5} fill="none" strokeLinecap="round" />
        </g>
      );
    case "content":
      return (
        <g transform="translate(100,74)">
          {blush}
          <circle cx="12" cy="-2" r="3.2" fill={INK} />
          <circle cx="13.1" cy="-3.2" r="0.9" fill="#fff" />
          <circle cx="-12" cy="-2" r="3.2" fill={INK} />
          <circle cx="-10.9" cy="-3.2" r="0.9" fill="#fff" />
          <path d="M-10,10 Q0,15 10,10" stroke={INK} strokeWidth={2.5} fill="none" strokeLinecap="round" />
        </g>
      );
    case "worried":
      return (
        <g transform="translate(100,74)">
          <path d="M-16,-11 L-6,-8" stroke={INK} strokeWidth={2.3} strokeLinecap="round" />
          <path d="M16,-11 L6,-8" stroke={INK} strokeWidth={2.3} strokeLinecap="round" />
          <ellipse cx="-11" cy="-1" rx="3" ry="4" fill={INK} />
          <ellipse cx="11" cy="-1" rx="3" ry="4" fill={INK} />
          <path d="M-8,12 Q0,9 8,12" stroke={INK} strokeWidth={2.5} fill="none" strokeLinecap="round" />
        </g>
      );
    case "wilted":
      return (
        <g transform="translate(100,74)">
          <path d="M-16,-2 Q-11,3 -6,-2" stroke={INK} strokeWidth={2.5} fill="none" strokeLinecap="round" />
          <path d="M6,-2 Q11,3 16,-2" stroke={INK} strokeWidth={2.5} fill="none" strokeLinecap="round" />
          <path d="M-11,15 Q0,8 11,15" stroke={INK} strokeWidth={2.5} fill="none" strokeLinecap="round" />
          <path
            className="mascot-droplet"
            d="M27,-16 C31,-10 31,-4 27,-2 C23,-4 23,-10 27,-16 Z"
            fill="#7EC1E8"
            stroke={INK}
            strokeWidth={1.5}
          />
        </g>
      );
  }
}

const PETAL_ANGLES = [0, 51, 103, 154, 206, 257, 309];

function Sparkle({ x, y, delay, size = 7 }: { x: number; y: number; delay: string; size?: number }) {
  return (
    <g className="mascot-sparkle" transform={`translate(${x},${y})`} style={{ animationDelay: delay }}>
      <path
        d={`M0,-${size} L${size * 0.32},-${size * 0.32} L${size},0 L${size * 0.32},${size * 0.32} L0,${size} L${-size * 0.32},${size * 0.32} L${-size},0 L${-size * 0.32},${-size * 0.32} Z`}
        fill="#FFD873"
        stroke={INK}
        strokeWidth={1}
      />
    </g>
  );
}

/** Little annoyed-mark cluster, shown near the head only while hovering a worried mascot. */
function AngerMark({ x, y }: { x: number; y: number }) {
  return (
    <g className="mascot-angry-mark" transform={`translate(${x},${y})`}>
      <path d="M-7,-6 L1,3" stroke="#D6473C" strokeWidth={2.2} strokeLinecap="round" />
      <path d="M-1,-8 L4,4" stroke="#D6473C" strokeWidth={2.2} strokeLinecap="round" />
      <path d="M5,-5 L8,5" stroke="#D6473C" strokeWidth={2.2} strokeLinecap="round" />
    </g>
  );
}

export function Mascot({ state, onClick }: Props) {
  const droopy = state === "worried" || state === "wilted";
  const petalFill = state === "wilted" ? "#F2E9DC" : "#FFFFFF";
  const petalShade = state === "wilted" ? "#E4D7C4" : "#EFE9DC";

  return (
    <svg
      className={`mascot-illustration mascot-${state}`}
      width={130}
      viewBox="0 0 200 190"
      style={{
        cursor: onClick ? "pointer" : "default",
        display: "block",
        overflow: "visible",
        // The container this sits in has pointer-events disabled so its
        // negative-margin overlap with the card below doesn't swallow
        // clicks meant for the title bar (see styles.css). visiblePainted
        // re-enables hover/click on just the drawn plant, not the empty
        // corners of this box, so that fix stays intact.
        pointerEvents: "visiblePainted",
      }}
      onClick={onClick}
      role="img"
      aria-label={`Plant mood: ${state}`}
    >
      <defs>
        <linearGradient id="mascot-pot" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stopColor="#CB8F5E" />
          <stop offset="55%" stopColor="#B8794F" />
          <stop offset="100%" stopColor="#9C6640" />
        </linearGradient>
        <linearGradient id="mascot-rim" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#E4B584" />
          <stop offset="100%" stopColor="#C98F5C" />
        </linearGradient>
        <linearGradient id="mascot-leaf" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stopColor="#7FBE68" />
          <stop offset="100%" stopColor="#5C9A4C" />
        </linearGradient>
        <radialGradient id="mascot-center" cx="38%" cy="35%" r="70%">
          <stop offset="0%" stopColor="#FFC85C" />
          <stop offset="70%" stopColor="#F6A623" />
          <stop offset="100%" stopColor="#DE8C14" />
        </radialGradient>
      </defs>

      {/* cast shadow to ground the pot against a transparent backdrop */}
      <ellipse cx="100" cy="174" rx="38" ry="6" fill="#000000" opacity={0.14} />

      {/* pot */}
      <path
        d="M65,119 L135,119 L127,168 Q100,174 73,168 Z"
        fill="url(#mascot-pot)"
        stroke={INK}
        strokeWidth={3}
        strokeLinejoin="round"
      />
      <path d="M70,137 Q100,142 130,137" stroke="#00000022" strokeWidth={2} fill="none" strokeLinecap="round" />
      <path d="M72,153 Q100,158 128,153" stroke="#00000022" strokeWidth={2} fill="none" strokeLinecap="round" />
      <path d="M74,127 Q70,146 76,163" stroke="#FFFFFF55" strokeWidth={5} fill="none" strokeLinecap="round" />

      {/* rim + soil */}
      <ellipse cx="100" cy="120" rx="35" ry="9" fill="url(#mascot-rim)" stroke={INK} strokeWidth={3} />
      <ellipse cx="100" cy="118" rx="27" ry="6" fill="#5B4632" />
      <circle cx="90" cy="117" r="1.6" fill="#40311F" />
      <circle cx="108" cy="119" r="1.4" fill="#40311F" />
      <circle cx="100" cy="115.5" r="1.3" fill="#40311F" />

      {/* stem */}
      <path d="M100,120 L100,90" stroke="#5C9950" strokeWidth={6} strokeLinecap="round" />

      {/* leaves */}
      <path
        d="M100,106 C86,100 78,104 68,98 C76,112 88,114 100,110 Z"
        fill="url(#mascot-leaf)"
        stroke={INK}
        strokeWidth={2.5}
        strokeLinejoin="round"
      />
      <path d="M96,105 C89,103 83,105 77,102" stroke="#3F7A34" strokeWidth={1.4} fill="none" strokeLinecap="round" />
      <path
        d="M100,102 C114,96 122,100 132,94 C124,108 112,110 100,106 Z"
        fill="url(#mascot-leaf)"
        stroke={INK}
        strokeWidth={2.5}
        strokeLinejoin="round"
      />
      <path d="M104,101 C111,99 117,101 123,98" stroke="#3F7A34" strokeWidth={1.4} fill="none" strokeLinecap="round" />

      {/* petals */}
      <g
        style={{
          transformOrigin: "100px 68px",
          transition: "transform 0.25s",
          transform: droopy ? "translateY(5px) scaleY(0.85)" : "translateY(0) scaleY(1)",
        }}
      >
        {PETAL_ANGLES.map((angle) => (
          <g key={angle} transform={`translate(100,68) rotate(${angle})`}>
            <ellipse
              cx="34"
              cy="0"
              rx="19"
              ry="13"
              fill={petalFill}
              stroke={INK}
              strokeWidth={2.5}
              style={{ transition: "fill 0.25s" }}
            />
            <ellipse cx="26" cy="0" rx="7" ry="8.5" fill={petalShade} opacity={0.7} style={{ transition: "fill 0.25s" }} />
          </g>
        ))}
      </g>

      {/* flower center */}
      <circle cx="100" cy="72" r="33" fill="url(#mascot-center)" stroke={INK} strokeWidth={3} />
      {[...Array(10)].map((_, i) => {
        const a = (i / 10) * Math.PI * 2;
        const r1 = 12;
        const r2 = 19;
        return (
          <line
            key={i}
            x1={100 + Math.cos(a) * r1}
            y1={72 + Math.sin(a) * r1}
            x2={100 + Math.cos(a) * r2}
            y2={72 + Math.sin(a) * r2}
            stroke="#C97A16"
            strokeWidth={1.4}
            strokeLinecap="round"
            opacity={0.4}
          />
        );
      })}

      <Face state={state} />

      <Sparkle x={50} y={28} delay="0s" size={7} />
      <Sparkle x={155} y={22} delay="0.15s" size={6} />
      <Sparkle x={102} y={2} delay="0.3s" size={5.5} />
      <AngerMark x={140} y={30} />
    </svg>
  );
}

/** Worst-first: overdue plants wilt the mascot even if others are fine. */
export function mascotStateForCounts(overdue: number, dueToday: number, soon: number): MascotState {
  if (overdue > 0) return "wilted";
  if (dueToday > 0) return "worried";
  if (soon > 0) return "content";
  return "happy";
}
