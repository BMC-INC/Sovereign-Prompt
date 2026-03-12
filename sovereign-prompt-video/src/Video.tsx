import React from "react";
import {
  AbsoluteFill,
  Sequence,
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  spring,
  Easing,
} from "remotion";

// ── Color palette ──
const COLORS = {
  bg: "#0a0a0f",
  bgCard: "#12121a",
  accent: "#6c5ce7",
  accentGlow: "#a29bfe",
  green: "#00cec9",
  orange: "#fdcb6e",
  red: "#ff7675",
  white: "#f5f6fa",
  gray: "#636e72",
  dim: "#2d3436",
};

// ── Shared styles ──
const centerFlex: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  flexDirection: "column",
};

// ── Scene 1: Title ──
const TitleScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const logoScale = spring({ frame, fps, config: { damping: 12, stiffness: 80 } });
  const titleOpacity = interpolate(frame, [15, 35], [0, 1], { extrapolateRight: "clamp" });
  const titleY = interpolate(frame, [15, 35], [40, 0], { extrapolateRight: "clamp" });
  const subtitleOpacity = interpolate(frame, [35, 55], [0, 1], { extrapolateRight: "clamp" });
  const lineWidth = interpolate(frame, [40, 70], [0, 400], { extrapolateRight: "clamp" });
  const badgeOpacity = interpolate(frame, [55, 70], [0, 1], { extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      {/* Subtle grid */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          backgroundImage: `linear-gradient(${COLORS.dim}22 1px, transparent 1px), linear-gradient(90deg, ${COLORS.dim}22 1px, transparent 1px)`,
          backgroundSize: "60px 60px",
        }}
      />

      {/* Glow orb */}
      <div
        style={{
          position: "absolute",
          width: 600,
          height: 600,
          borderRadius: "50%",
          background: `radial-gradient(circle, ${COLORS.accent}30, transparent 70%)`,
          transform: `scale(${logoScale})`,
          filter: "blur(80px)",
        }}
      />

      {/* Logo icon */}
      <div
        style={{
          fontSize: 90,
          transform: `scale(${logoScale})`,
          marginBottom: 20,
          filter: `drop-shadow(0 0 40px ${COLORS.accent}88)`,
        }}
      >
        ⚡
      </div>

      {/* Title */}
      <div
        style={{
          fontSize: 88,
          fontWeight: 800,
          fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif",
          color: COLORS.white,
          opacity: titleOpacity,
          transform: `translateY(${titleY}px)`,
          letterSpacing: -2,
        }}
      >
        Sovereign<span style={{ color: COLORS.accent }}>Prompt</span>
      </div>

      {/* Accent line */}
      <div
        style={{
          width: lineWidth,
          height: 3,
          background: `linear-gradient(90deg, transparent, ${COLORS.accent}, transparent)`,
          marginTop: 16,
          marginBottom: 20,
          borderRadius: 2,
        }}
      />

      {/* Subtitle */}
      <div
        style={{
          fontSize: 32,
          fontFamily: "SF Mono, Menlo, monospace",
          color: COLORS.gray,
          opacity: subtitleOpacity,
          letterSpacing: 1,
        }}
      >
        MCP Prompt Optimization Engine
      </div>

      {/* Badge */}
      <div
        style={{
          marginTop: 30,
          opacity: badgeOpacity,
          padding: "10px 28px",
          borderRadius: 20,
          border: `1px solid ${COLORS.accent}44`,
          background: `${COLORS.accent}15`,
          fontSize: 20,
          fontFamily: "SF Mono, Menlo, monospace",
          color: COLORS.accentGlow,
        }}
      >
        Built by ExecLayer Inc.
      </div>
    </AbsoluteFill>
  );
};

// ── Scene 2: The Problem ──
const ProblemScene: React.FC = () => {
  const frame = useCurrentFrame();

  const headerOpacity = interpolate(frame, [0, 15], [0, 1], { extrapolateRight: "clamp" });

  const wastefulPrompt = `"Hey, could you please kindly help me out and maybe\npossibly write something that sort of fixes the thing\nwith the stuff? Thank you so much! Also additionally\ncan you do the other thing as well as the rest of it?"`;

  const promptOpacity = interpolate(frame, [15, 30], [0, 1], { extrapolateRight: "clamp" });
  const promptScale = interpolate(frame, [15, 30], [0.95, 1], { extrapolateRight: "clamp" });

  const issues = [
    { label: "Politeness tokens", color: COLORS.orange, delay: 35 },
    { label: "Vague language", color: COLORS.red, delay: 45 },
    { label: "Multiple tasks", color: COLORS.red, delay: 55 },
    { label: "No output format", color: COLORS.orange, delay: 65 },
    { label: "127 tokens wasted", color: COLORS.red, delay: 75 },
  ];

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div
        style={{
          fontSize: 48,
          fontWeight: 700,
          fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif",
          color: COLORS.red,
          opacity: headerOpacity,
          marginBottom: 40,
        }}
      >
        The Problem: Token Waste
      </div>

      <div
        style={{
          background: COLORS.bgCard,
          border: `1px solid ${COLORS.red}44`,
          borderRadius: 16,
          padding: "36px 48px",
          maxWidth: 1100,
          opacity: promptOpacity,
          transform: `scale(${promptScale})`,
        }}
      >
        <div
          style={{
            fontSize: 22,
            fontFamily: "SF Mono, Menlo, monospace",
            color: COLORS.gray,
            lineHeight: 1.7,
            whiteSpace: "pre-wrap",
          }}
        >
          {wastefulPrompt}
        </div>
      </div>

      <div
        style={{
          display: "flex",
          gap: 16,
          marginTop: 40,
          flexWrap: "wrap",
          justifyContent: "center",
        }}
      >
        {issues.map((issue, i) => {
          const opacity = interpolate(frame, [issue.delay, issue.delay + 10], [0, 1], {
            extrapolateRight: "clamp",
            extrapolateLeft: "clamp",
          });
          const y = interpolate(frame, [issue.delay, issue.delay + 10], [20, 0], {
            extrapolateRight: "clamp",
            extrapolateLeft: "clamp",
          });
          return (
            <div
              key={i}
              style={{
                padding: "10px 24px",
                borderRadius: 12,
                background: `${issue.color}18`,
                border: `1px solid ${issue.color}55`,
                color: issue.color,
                fontSize: 20,
                fontFamily: "SF Mono, Menlo, monospace",
                fontWeight: 600,
                opacity,
                transform: `translateY(${y}px)`,
              }}
            >
              {issue.label}
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
};

// ── Scene 3: The Solution ──
const SolutionScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const headerOpacity = interpolate(frame, [0, 15], [0, 1], { extrapolateRight: "clamp" });

  const before = `Please kindly help me fix the thing with the stuff`;
  const after = `Fix the authentication middleware null pointer\non line 42 of auth.rs. Return JSON error response.`;

  const beforeOpacity = interpolate(frame, [10, 25], [0, 1], { extrapolateRight: "clamp" });
  const arrowOpacity = interpolate(frame, [30, 40], [0, 1], { extrapolateRight: "clamp" });
  const afterOpacity = interpolate(frame, [40, 55], [0, 1], { extrapolateRight: "clamp" });
  const afterGlow = interpolate(frame, [55, 75], [0, 1], { extrapolateRight: "clamp" });

  const savingsOpacity = interpolate(frame, [60, 75], [0, 1], { extrapolateRight: "clamp" });
  const savingsScale = spring({ frame: Math.max(0, frame - 60), fps, config: { damping: 10, stiffness: 100 } });

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div
        style={{
          fontSize: 48,
          fontWeight: 700,
          fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif",
          color: COLORS.green,
          opacity: headerOpacity,
          marginBottom: 50,
        }}
      >
        ⚡ SovereignPrompt Optimizes
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 50 }}>
        {/* Before */}
        <div
          style={{
            background: COLORS.bgCard,
            border: `1px solid ${COLORS.red}44`,
            borderRadius: 16,
            padding: "30px 36px",
            width: 520,
            opacity: beforeOpacity,
          }}
        >
          <div
            style={{
              fontSize: 14,
              fontFamily: "SF Mono, Menlo, monospace",
              color: COLORS.red,
              marginBottom: 12,
              textTransform: "uppercase",
              letterSpacing: 2,
            }}
          >
            Before — 14 tokens
          </div>
          <div
            style={{
              fontSize: 22,
              fontFamily: "SF Mono, Menlo, monospace",
              color: COLORS.gray,
              lineHeight: 1.6,
            }}
          >
            {before}
          </div>
        </div>

        {/* Arrow */}
        <div
          style={{
            fontSize: 60,
            color: COLORS.accent,
            opacity: arrowOpacity,
            filter: `drop-shadow(0 0 20px ${COLORS.accent}66)`,
          }}
        >
          →
        </div>

        {/* After */}
        <div
          style={{
            background: COLORS.bgCard,
            border: `1px solid ${COLORS.green}44`,
            borderRadius: 16,
            padding: "30px 36px",
            width: 520,
            opacity: afterOpacity,
            boxShadow: `0 0 ${afterGlow * 40}px ${COLORS.green}22`,
          }}
        >
          <div
            style={{
              fontSize: 14,
              fontFamily: "SF Mono, Menlo, monospace",
              color: COLORS.green,
              marginBottom: 12,
              textTransform: "uppercase",
              letterSpacing: 2,
            }}
          >
            After — 18 tokens
          </div>
          <div
            style={{
              fontSize: 22,
              fontFamily: "SF Mono, Menlo, monospace",
              color: COLORS.white,
              lineHeight: 1.6,
              whiteSpace: "pre-wrap",
            }}
          >
            {after}
          </div>
        </div>
      </div>

      {/* Savings badge */}
      <div
        style={{
          marginTop: 50,
          opacity: savingsOpacity,
          transform: `scale(${savingsScale})`,
          padding: "16px 48px",
          borderRadius: 16,
          background: `linear-gradient(135deg, ${COLORS.green}22, ${COLORS.accent}22)`,
          border: `1px solid ${COLORS.green}44`,
        }}
      >
        <span
          style={{
            fontSize: 36,
            fontWeight: 800,
            fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif",
            color: COLORS.green,
          }}
        >
          Precise. Cheaper. Better output.
        </span>
      </div>
    </AbsoluteFill>
  );
};

// ── Scene 4: Four MCP Tools ──
const ToolsScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const headerOpacity = interpolate(frame, [0, 15], [0, 1], { extrapolateRight: "clamp" });

  const tools = [
    {
      name: "optimize_prompt",
      desc: "Analyze, refine, and generate\n3 prompt variants",
      icon: "🔬",
      color: COLORS.accent,
    },
    {
      name: "capture_output",
      desc: "Store AI responses for\nlearning & analytics",
      icon: "📡",
      color: COLORS.green,
    },
    {
      name: "get_stats",
      desc: "Token savings, top issues,\nand usage metrics",
      icon: "📊",
      color: COLORS.orange,
    },
    {
      name: "get_history",
      desc: "Full prompt history\nwith pagination",
      icon: "🗂️",
      color: COLORS.accentGlow,
    },
  ];

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div
        style={{
          fontSize: 48,
          fontWeight: 700,
          fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif",
          color: COLORS.white,
          opacity: headerOpacity,
          marginBottom: 60,
        }}
      >
        4 MCP Tools
      </div>

      <div style={{ display: "flex", gap: 30 }}>
        {tools.map((tool, i) => {
          const delay = 15 + i * 15;
          const cardScale = spring({
            frame: Math.max(0, frame - delay),
            fps,
            config: { damping: 12, stiffness: 80 },
          });
          const opacity = interpolate(frame, [delay, delay + 10], [0, 1], {
            extrapolateRight: "clamp",
            extrapolateLeft: "clamp",
          });

          return (
            <div
              key={i}
              style={{
                width: 360,
                padding: "40px 30px",
                background: COLORS.bgCard,
                border: `1px solid ${tool.color}33`,
                borderRadius: 20,
                opacity,
                transform: `scale(${cardScale})`,
                ...centerFlex,
              }}
            >
              <div style={{ fontSize: 52, marginBottom: 20 }}>{tool.icon}</div>
              <div
                style={{
                  fontSize: 22,
                  fontFamily: "SF Mono, Menlo, monospace",
                  color: tool.color,
                  fontWeight: 700,
                  marginBottom: 14,
                }}
              >
                {tool.name}
              </div>
              <div
                style={{
                  fontSize: 18,
                  fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif",
                  color: COLORS.gray,
                  textAlign: "center",
                  lineHeight: 1.5,
                  whiteSpace: "pre-wrap",
                }}
              >
                {tool.desc}
              </div>
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
};

// ── Scene 5: Analysis Heuristics ──
const AnalysisScene: React.FC = () => {
  const frame = useCurrentFrame();

  const headerOpacity = interpolate(frame, [0, 15], [0, 1], { extrapolateRight: "clamp" });

  const checks = [
    { label: "Vagueness Detection", icon: "🔍" },
    { label: "Redundancy Analysis", icon: "♻️" },
    { label: "Missing Context", icon: "📋" },
    { label: "Politeness Tokens", icon: "🎩" },
    { label: "Prompt Injection", icon: "🛡️" },
    { label: "Task Separation", icon: "✂️" },
    { label: "Output Format", icon: "📐" },
    { label: "Ambiguous Pronouns", icon: "👤" },
  ];

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div
        style={{
          fontSize: 48,
          fontWeight: 700,
          fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif",
          color: COLORS.white,
          opacity: headerOpacity,
          marginBottom: 50,
        }}
      >
        8 Heuristic Checks
      </div>

      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          gap: 20,
          maxWidth: 1200,
          justifyContent: "center",
        }}
      >
        {checks.map((check, i) => {
          const delay = 10 + i * 8;
          const opacity = interpolate(frame, [delay, delay + 12], [0, 1], {
            extrapolateRight: "clamp",
            extrapolateLeft: "clamp",
          });
          const x = interpolate(frame, [delay, delay + 12], [-30, 0], {
            extrapolateRight: "clamp",
            extrapolateLeft: "clamp",
          });

          return (
            <div
              key={i}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 14,
                padding: "18px 30px",
                background: COLORS.bgCard,
                border: `1px solid ${COLORS.accent}33`,
                borderRadius: 14,
                opacity,
                transform: `translateX(${x}px)`,
                width: 340,
              }}
            >
              <span style={{ fontSize: 32 }}>{check.icon}</span>
              <span
                style={{
                  fontSize: 22,
                  fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif",
                  color: COLORS.white,
                  fontWeight: 600,
                }}
              >
                {check.label}
              </span>
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
};

// ── Scene 6: Tech Stack ──
const TechScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const headerOpacity = interpolate(frame, [0, 15], [0, 1], { extrapolateRight: "clamp" });

  const stack = [
    { name: "Rust", detail: "Zero-cost abstractions", color: COLORS.orange },
    { name: "rmcp 0.1", detail: "Native MCP transport", color: COLORS.accent },
    { name: "tiktoken", detail: "cl100k_base encoding", color: COLORS.green },
    { name: "SQLite", detail: "Embedded persistence", color: COLORS.accentGlow },
    { name: "tokio", detail: "Async runtime", color: COLORS.orange },
  ];

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div
        style={{
          fontSize: 48,
          fontWeight: 700,
          fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif",
          color: COLORS.white,
          opacity: headerOpacity,
          marginBottom: 50,
        }}
      >
        Production Stack
      </div>

      {stack.map((item, i) => {
        const delay = 15 + i * 12;
        const width = interpolate(frame, [delay, delay + 15], [0, 900], {
          extrapolateRight: "clamp",
          extrapolateLeft: "clamp",
        });
        const opacity = interpolate(frame, [delay, delay + 10], [0, 1], {
          extrapolateRight: "clamp",
          extrapolateLeft: "clamp",
        });

        return (
          <div
            key={i}
            style={{
              display: "flex",
              alignItems: "center",
              marginBottom: 16,
              opacity,
              width: 900,
            }}
          >
            <div
              style={{
                height: 56,
                width,
                background: `linear-gradient(90deg, ${item.color}33, ${item.color}08)`,
                borderRadius: 12,
                border: `1px solid ${item.color}44`,
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                padding: "0 30px",
                overflow: "hidden",
              }}
            >
              <span
                style={{
                  fontSize: 24,
                  fontFamily: "SF Mono, Menlo, monospace",
                  fontWeight: 700,
                  color: item.color,
                  whiteSpace: "nowrap",
                }}
              >
                {item.name}
              </span>
              <span
                style={{
                  fontSize: 20,
                  fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif",
                  color: COLORS.gray,
                  whiteSpace: "nowrap",
                }}
              >
                {item.detail}
              </span>
            </div>
          </div>
        );
      })}
    </AbsoluteFill>
  );
};

// ── Scene 7: Closing ──
const ClosingScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const logoScale = spring({ frame, fps, config: { damping: 12, stiffness: 80 } });
  const titleOpacity = interpolate(frame, [10, 25], [0, 1], { extrapolateRight: "clamp" });
  const ctaOpacity = interpolate(frame, [30, 45], [0, 1], { extrapolateRight: "clamp" });
  const pulseGlow = Math.sin(frame * 0.08) * 0.3 + 0.7;

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      {/* Glow */}
      <div
        style={{
          position: "absolute",
          width: 800,
          height: 800,
          borderRadius: "50%",
          background: `radial-gradient(circle, ${COLORS.accent}20, transparent 70%)`,
          filter: "blur(100px)",
          opacity: pulseGlow,
        }}
      />

      <div
        style={{
          fontSize: 70,
          transform: `scale(${logoScale})`,
          marginBottom: 20,
          filter: `drop-shadow(0 0 40px ${COLORS.accent}88)`,
        }}
      >
        ⚡
      </div>

      <div
        style={{
          fontSize: 72,
          fontWeight: 800,
          fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif",
          color: COLORS.white,
          opacity: titleOpacity,
          letterSpacing: -2,
        }}
      >
        Sovereign<span style={{ color: COLORS.accent }}>Prompt</span>
      </div>

      <div
        style={{
          marginTop: 30,
          opacity: ctaOpacity,
          fontSize: 28,
          fontFamily: "SF Mono, Menlo, monospace",
          color: COLORS.gray,
        }}
      >
        Stop wasting tokens. Start shipping precision.
      </div>

      <div
        style={{
          marginTop: 40,
          opacity: ctaOpacity,
          padding: "14px 44px",
          borderRadius: 14,
          background: `linear-gradient(135deg, ${COLORS.accent}, ${COLORS.green})`,
          fontSize: 26,
          fontWeight: 700,
          fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif",
          color: COLORS.white,
          boxShadow: `0 0 ${pulseGlow * 30}px ${COLORS.accent}66`,
        }}
      >
        cargo build --release
      </div>

      <div
        style={{
          marginTop: 30,
          opacity: ctaOpacity,
          fontSize: 20,
          fontFamily: "SF Mono, Menlo, monospace",
          color: COLORS.dim,
        }}
      >
        ExecLayer Inc. — v0.1.0
      </div>
    </AbsoluteFill>
  );
};

// ── Main composition ──
export const SovereignPromptVideo: React.FC = () => {
  return (
    <AbsoluteFill style={{ background: COLORS.bg }}>
      <Sequence from={0} durationInFrames={90}>
        <TitleScene />
      </Sequence>
      <Sequence from={90} durationInFrames={90}>
        <ProblemScene />
      </Sequence>
      <Sequence from={180} durationInFrames={90}>
        <SolutionScene />
      </Sequence>
      <Sequence from={270} durationInFrames={75}>
        <ToolsScene />
      </Sequence>
      <Sequence from={345} durationInFrames={75}>
        <AnalysisScene />
      </Sequence>
      <Sequence from={420} durationInFrames={75}>
        <TechScene />
      </Sequence>
      <Sequence from={495} durationInFrames={75}>
        <ClosingScene />
      </Sequence>
    </AbsoluteFill>
  );
};
