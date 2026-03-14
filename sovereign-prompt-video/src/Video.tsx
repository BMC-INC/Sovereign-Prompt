import React from "react";
import {
  AbsoluteFill,
  Audio,
  Sequence,
  staticFile,
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
  const taglineOpacity = interpolate(frame, [80, 100], [0, 1], { extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div
        style={{
          position: "absolute",
          inset: 0,
          backgroundImage: `linear-gradient(${COLORS.dim}22 1px, transparent 1px), linear-gradient(90deg, ${COLORS.dim}22 1px, transparent 1px)`,
          backgroundSize: "60px 60px",
        }}
      />
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
      <div style={{ fontSize: 90, transform: `scale(${logoScale})`, marginBottom: 20, filter: `drop-shadow(0 0 40px ${COLORS.accent}88)` }}>
        ⚡
      </div>
      <div style={{ fontSize: 88, fontWeight: 800, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.white, opacity: titleOpacity, transform: `translateY(${titleY}px)`, letterSpacing: -2 }}>
        Sovereign<span style={{ color: COLORS.accent }}>Prompt</span>
      </div>
      <div style={{ width: lineWidth, height: 3, background: `linear-gradient(90deg, transparent, ${COLORS.accent}, transparent)`, marginTop: 16, marginBottom: 20, borderRadius: 2 }} />
      <div style={{ fontSize: 32, fontFamily: "SF Mono, Menlo, monospace", color: COLORS.gray, opacity: subtitleOpacity, letterSpacing: 1 }}>
        MCP-Native Prompt Optimization Engine
      </div>
      <div style={{ marginTop: 30, opacity: badgeOpacity, padding: "10px 28px", borderRadius: 20, border: `1px solid ${COLORS.accent}44`, background: `${COLORS.accent}15`, fontSize: 20, fontFamily: "SF Mono, Menlo, monospace", color: COLORS.accentGlow }}>
        Built in Pure Rust — Zero Unsafe Code
      </div>
      <div style={{ marginTop: 20, opacity: taglineOpacity, fontSize: 24, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.green }}>
        Your prompts are bleeding tokens. We stop the bleed.
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
    { label: "Redundant phrasing", color: COLORS.red, delay: 55 },
    { label: "Multiple tasks bundled", color: COLORS.orange, delay: 65 },
    { label: "No output format", color: COLORS.orange, delay: 75 },
    { label: "50+ tokens wasted", color: COLORS.red, delay: 85 },
  ];

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div style={{ fontSize: 48, fontWeight: 700, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.red, opacity: headerOpacity, marginBottom: 40 }}>
        The Problem: Token Waste
      </div>
      <div style={{ background: COLORS.bgCard, border: `1px solid ${COLORS.red}44`, borderRadius: 16, padding: "36px 48px", maxWidth: 1100, opacity: promptOpacity, transform: `scale(${promptScale})` }}>
        <div style={{ fontSize: 22, fontFamily: "SF Mono, Menlo, monospace", color: COLORS.gray, lineHeight: 1.7, whiteSpace: "pre-wrap" }}>
          {wastefulPrompt}
        </div>
      </div>
      <div style={{ display: "flex", gap: 16, marginTop: 40, flexWrap: "wrap", justifyContent: "center" }}>
        {issues.map((issue, i) => {
          const opacity = interpolate(frame, [issue.delay, issue.delay + 10], [0, 1], { extrapolateRight: "clamp", extrapolateLeft: "clamp" });
          const y = interpolate(frame, [issue.delay, issue.delay + 10], [20, 0], { extrapolateRight: "clamp", extrapolateLeft: "clamp" });
          return (
            <div key={i} style={{ padding: "10px 24px", borderRadius: 12, background: `${issue.color}18`, border: `1px solid ${issue.color}55`, color: issue.color, fontSize: 20, fontFamily: "SF Mono, Menlo, monospace", fontWeight: 600, opacity, transform: `translateY(${y}px)` }}>
              {issue.label}
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
};

// ── Scene 3: 9 Heuristic Checks ──
const AnalysisScene: React.FC = () => {
  const frame = useCurrentFrame();

  const headerOpacity = interpolate(frame, [0, 15], [0, 1], { extrapolateRight: "clamp" });

  const checks = [
    { label: "Vagueness Detection", icon: "🔍", color: COLORS.orange },
    { label: "Redundancy Analysis", icon: "♻️", color: COLORS.orange },
    { label: "Missing Context", icon: "📋", color: COLORS.red },
    { label: "Politeness Tokens", icon: "🎩", color: COLORS.orange },
    { label: "Prompt Injection", icon: "🛡️", color: COLORS.red },
    { label: "Task Separation", icon: "✂️", color: COLORS.orange },
    { label: "Output Format", icon: "📐", color: COLORS.orange },
    { label: "Ambiguous Pronouns", icon: "👤", color: COLORS.orange },
    { label: "Governance Policy", icon: "🔒", color: COLORS.red },
  ];

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div style={{ fontSize: 48, fontWeight: 700, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.white, opacity: headerOpacity, marginBottom: 50 }}>
        9 Heuristic Checks
      </div>
      <div style={{ display: "flex", flexWrap: "wrap", gap: 18, maxWidth: 1200, justifyContent: "center" }}>
        {checks.map((check, i) => {
          const delay = 10 + i * 7;
          const opacity = interpolate(frame, [delay, delay + 12], [0, 1], { extrapolateRight: "clamp", extrapolateLeft: "clamp" });
          const x = interpolate(frame, [delay, delay + 12], [-30, 0], { extrapolateRight: "clamp", extrapolateLeft: "clamp" });
          return (
            <div key={i} style={{ display: "flex", alignItems: "center", gap: 14, padding: "16px 26px", background: COLORS.bgCard, border: `1px solid ${check.color}33`, borderRadius: 14, opacity, transform: `translateX(${x}px)`, width: 340 }}>
              <span style={{ fontSize: 30 }}>{check.icon}</span>
              <span style={{ fontSize: 20, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.white, fontWeight: 600 }}>
                {check.label}
              </span>
            </div>
          );
        })}
      </div>
      <div style={{ marginTop: 30, opacity: interpolate(frame, [80, 95], [0, 1], { extrapolateRight: "clamp" }), fontSize: 22, fontFamily: "SF Mono, Menlo, monospace", color: COLORS.green }}>
        + Custom heuristic plugins via TOML config
      </div>
    </AbsoluteFill>
  );
};

// ── Scene 4: The Solution (Before/After) ──
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
  const variantsOpacity = interpolate(frame, [80, 95], [0, 1], { extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div style={{ fontSize: 48, fontWeight: 700, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.green, opacity: headerOpacity, marginBottom: 50 }}>
        ⚡ Optimized In Microseconds
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 50 }}>
        <div style={{ background: COLORS.bgCard, border: `1px solid ${COLORS.red}44`, borderRadius: 16, padding: "30px 36px", width: 520, opacity: beforeOpacity }}>
          <div style={{ fontSize: 14, fontFamily: "SF Mono, Menlo, monospace", color: COLORS.red, marginBottom: 12, textTransform: "uppercase", letterSpacing: 2 }}>Before</div>
          <div style={{ fontSize: 22, fontFamily: "SF Mono, Menlo, monospace", color: COLORS.gray, lineHeight: 1.6 }}>{before}</div>
        </div>
        <div style={{ fontSize: 60, color: COLORS.accent, opacity: arrowOpacity, filter: `drop-shadow(0 0 20px ${COLORS.accent}66)` }}>→</div>
        <div style={{ background: COLORS.bgCard, border: `1px solid ${COLORS.green}44`, borderRadius: 16, padding: "30px 36px", width: 520, opacity: afterOpacity, boxShadow: `0 0 ${afterGlow * 40}px ${COLORS.green}22` }}>
          <div style={{ fontSize: 14, fontFamily: "SF Mono, Menlo, monospace", color: COLORS.green, marginBottom: 12, textTransform: "uppercase", letterSpacing: 2 }}>After</div>
          <div style={{ fontSize: 22, fontFamily: "SF Mono, Menlo, monospace", color: COLORS.white, lineHeight: 1.6, whiteSpace: "pre-wrap" }}>{after}</div>
        </div>
      </div>
      <div style={{ marginTop: 40, opacity: savingsOpacity, transform: `scale(${savingsScale})`, padding: "16px 48px", borderRadius: 16, background: `linear-gradient(135deg, ${COLORS.green}22, ${COLORS.accent}22)`, border: `1px solid ${COLORS.green}44` }}>
        <span style={{ fontSize: 36, fontWeight: 800, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.green }}>
          3 Variants: Precision • Creative • Concise
        </span>
      </div>
      <div style={{ marginTop: 16, display: "flex", gap: 16, opacity: variantsOpacity }}>
        <span style={{ padding: "6px 16px", borderRadius: 8, background: `${COLORS.accent}22`, border: `1px solid ${COLORS.accent}44`, color: COLORS.accentGlow, fontSize: 18, fontFamily: "SF Mono, Menlo, monospace" }}>4 tokenizer models</span>
        <span style={{ padding: "6px 16px", borderRadius: 8, background: `${COLORS.green}22`, border: `1px solid ${COLORS.green}44`, color: COLORS.green, fontSize: 18, fontFamily: "SF Mono, Menlo, monospace" }}>7 domain templates</span>
      </div>
    </AbsoluteFill>
  );
};

// ── Scene 5: 15 MCP Tools ──
const ToolsScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const headerOpacity = interpolate(frame, [0, 15], [0, 1], { extrapolateRight: "clamp" });

  const tools = [
    { name: "optimize_prompt", color: COLORS.accent },
    { name: "capture_output", color: COLORS.green },
    { name: "get_stats", color: COLORS.orange },
    { name: "get_history", color: COLORS.accentGlow },
    { name: "list_templates", color: COLORS.accent },
    { name: "count_tokens", color: COLORS.green },
    { name: "governance_check", color: COLORS.red },
    { name: "governance_approve", color: COLORS.red },
    { name: "get_audit_trail", color: COLORS.orange },
    { name: "sign_optimization", color: COLORS.accentGlow },
    { name: "verify_signature", color: COLORS.accentGlow },
    { name: "savings_report", color: COLORS.green },
    { name: "rate_optimization", color: COLORS.orange },
    { name: "learning_insights", color: COLORS.accent },
    { name: "team_report", color: COLORS.green },
  ];

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div style={{ fontSize: 48, fontWeight: 700, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.white, opacity: headerOpacity, marginBottom: 40 }}>
        15 MCP Tools
      </div>
      <div style={{ display: "flex", flexWrap: "wrap", gap: 12, maxWidth: 1400, justifyContent: "center" }}>
        {tools.map((tool, i) => {
          const delay = 10 + i * 4;
          const opacity = interpolate(frame, [delay, delay + 8], [0, 1], { extrapolateRight: "clamp", extrapolateLeft: "clamp" });
          const scale = interpolate(frame, [delay, delay + 8], [0.9, 1], { extrapolateRight: "clamp", extrapolateLeft: "clamp" });
          return (
            <div key={i} style={{ padding: "12px 22px", background: COLORS.bgCard, border: `1px solid ${tool.color}44`, borderRadius: 12, opacity, transform: `scale(${scale})` }}>
              <span style={{ fontSize: 18, fontFamily: "SF Mono, Menlo, monospace", color: tool.color, fontWeight: 600 }}>{tool.name}</span>
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
};

// ── Scene 6: Security & Crypto ──
const SecurityScene: React.FC = () => {
  const frame = useCurrentFrame();

  const headerOpacity = interpolate(frame, [0, 15], [0, 1], { extrapolateRight: "clamp" });

  const features = [
    { label: "SHA-256 Content Hashing", desc: "Every optimization tamper-evident", color: COLORS.accent, delay: 15 },
    { label: "HMAC-SHA256 Signing", desc: "Non-repudiable audit trails", color: COLORS.green, delay: 30 },
    { label: "Governance Engine", desc: "SSN, credit cards, PII detection", color: COLORS.red, delay: 45 },
    { label: "Zero Cloud / Zero Telemetry", desc: "Nothing leaves your machine", color: COLORS.orange, delay: 60 },
    { label: "#![deny(unsafe_code)]", desc: "No escape hatches in the entire crate", color: COLORS.accentGlow, delay: 75 },
  ];

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div style={{ fontSize: 48, fontWeight: 700, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.white, opacity: headerOpacity, marginBottom: 50 }}>
        🔒 Security & Cryptographic Audit Trails
      </div>
      {features.map((feat, i) => {
        const opacity = interpolate(frame, [feat.delay, feat.delay + 12], [0, 1], { extrapolateRight: "clamp", extrapolateLeft: "clamp" });
        const x = interpolate(frame, [feat.delay, feat.delay + 12], [40, 0], { extrapolateRight: "clamp", extrapolateLeft: "clamp" });
        return (
          <div key={i} style={{ display: "flex", alignItems: "center", gap: 20, marginBottom: 16, opacity, transform: `translateX(${x}px)`, width: 900 }}>
            <div style={{ width: 12, height: 12, borderRadius: "50%", background: feat.color, boxShadow: `0 0 12px ${feat.color}88`, flexShrink: 0 }} />
            <div>
              <div style={{ fontSize: 24, fontFamily: "SF Mono, Menlo, monospace", color: feat.color, fontWeight: 700 }}>{feat.label}</div>
              <div style={{ fontSize: 18, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.gray, marginTop: 4 }}>{feat.desc}</div>
            </div>
          </div>
        );
      })}
    </AbsoluteFill>
  );
};

// ── Scene 7: Learning + Team Analytics ──
const LearningScene: React.FC = () => {
  const frame = useCurrentFrame();

  const headerOpacity = interpolate(frame, [0, 15], [0, 1], { extrapolateRight: "clamp" });

  const features = [
    { label: "Rate Optimizations", desc: "Thumbs up/down on every refinement", color: COLORS.green, delay: 15 },
    { label: "Learning Insights", desc: "Best domains, satisfaction rate, recommendations", color: COLORS.accent, delay: 35 },
    { label: "Team Reports", desc: "Aggregate savings across your organization", color: COLORS.orange, delay: 55 },
    { label: "Cost Estimates", desc: "Claude Sonnet 4 / Opus 4 / GPT-4o / GPT-4o-mini", color: COLORS.accentGlow, delay: 75 },
  ];

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div style={{ fontSize: 48, fontWeight: 700, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.white, opacity: headerOpacity, marginBottom: 50 }}>
        📊 Learning Loop + Team Analytics
      </div>
      {features.map((feat, i) => {
        const opacity = interpolate(frame, [feat.delay, feat.delay + 15], [0, 1], { extrapolateRight: "clamp", extrapolateLeft: "clamp" });
        return (
          <div key={i} style={{ display: "flex", alignItems: "center", gap: 20, marginBottom: 24, opacity, width: 800 }}>
            <div style={{ width: 48, height: 48, borderRadius: 12, background: `${feat.color}22`, border: `1px solid ${feat.color}44`, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 24, color: feat.color, fontWeight: 800, flexShrink: 0 }}>{i + 1}</div>
            <div>
              <div style={{ fontSize: 26, fontFamily: "SF Mono, Menlo, monospace", color: feat.color, fontWeight: 700 }}>{feat.label}</div>
              <div style={{ fontSize: 18, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.gray, marginTop: 4 }}>{feat.desc}</div>
            </div>
          </div>
        );
      })}
    </AbsoluteFill>
  );
};

// ── Scene 8: Tech Stack ──
const TechScene: React.FC = () => {
  const frame = useCurrentFrame();

  const headerOpacity = interpolate(frame, [0, 15], [0, 1], { extrapolateRight: "clamp" });

  const stack = [
    { name: "Rust + #![deny(unsafe)]", detail: "Pure safe Rust, zero-cost abstractions", color: COLORS.orange },
    { name: "rmcp 0.1", detail: "Native MCP — stdio + SSE transport", color: COLORS.accent },
    { name: "tiktoken-rs", detail: "4 tokenizer models simultaneously", color: COLORS.green },
    { name: "SQLite + sqlx", detail: "Async persistence, zero compile-time DB", color: COLORS.accentGlow },
    { name: "axum + WebSocket", detail: "Live dashboard with real-time analytics", color: COLORS.orange },
    { name: "sha2 + hmac", detail: "SHA-256 hashing, HMAC-SHA256 signing", color: COLORS.accent },
    { name: "toml", detail: "Fully configurable heuristics + plugins", color: COLORS.green },
  ];

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div style={{ fontSize: 48, fontWeight: 700, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.white, opacity: headerOpacity, marginBottom: 40 }}>
        Production Stack
      </div>
      {stack.map((item, i) => {
        const delay = 12 + i * 10;
        const width = interpolate(frame, [delay, delay + 12], [0, 900], { extrapolateRight: "clamp", extrapolateLeft: "clamp" });
        const opacity = interpolate(frame, [delay, delay + 8], [0, 1], { extrapolateRight: "clamp", extrapolateLeft: "clamp" });
        return (
          <div key={i} style={{ display: "flex", alignItems: "center", marginBottom: 12, opacity, width: 900 }}>
            <div style={{ height: 50, width, background: `linear-gradient(90deg, ${item.color}33, ${item.color}08)`, borderRadius: 12, border: `1px solid ${item.color}44`, display: "flex", alignItems: "center", justifyContent: "space-between", padding: "0 28px", overflow: "hidden" }}>
              <span style={{ fontSize: 22, fontFamily: "SF Mono, Menlo, monospace", fontWeight: 700, color: item.color, whiteSpace: "nowrap" }}>{item.name}</span>
              <span style={{ fontSize: 18, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.gray, whiteSpace: "nowrap" }}>{item.detail}</span>
            </div>
          </div>
        );
      })}
    </AbsoluteFill>
  );
};

// ── Scene 9: Closing ──
const ClosingScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const logoScale = spring({ frame, fps, config: { damping: 12, stiffness: 80 } });
  const titleOpacity = interpolate(frame, [10, 25], [0, 1], { extrapolateRight: "clamp" });
  const ctaOpacity = interpolate(frame, [30, 45], [0, 1], { extrapolateRight: "clamp" });
  const statsOpacity = interpolate(frame, [50, 65], [0, 1], { extrapolateRight: "clamp" });
  const pulseGlow = Math.sin(frame * 0.08) * 0.3 + 0.7;

  return (
    <AbsoluteFill style={{ ...centerFlex, background: COLORS.bg }}>
      <div style={{ position: "absolute", width: 800, height: 800, borderRadius: "50%", background: `radial-gradient(circle, ${COLORS.accent}20, transparent 70%)`, filter: "blur(100px)", opacity: pulseGlow }} />
      <div style={{ fontSize: 70, transform: `scale(${logoScale})`, marginBottom: 20, filter: `drop-shadow(0 0 40px ${COLORS.accent}88)` }}>⚡</div>
      <div style={{ fontSize: 72, fontWeight: 800, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.white, opacity: titleOpacity, letterSpacing: -2 }}>
        Sovereign<span style={{ color: COLORS.accent }}>Prompt</span>
      </div>
      <div style={{ marginTop: 24, opacity: ctaOpacity, fontSize: 28, fontFamily: "SF Mono, Menlo, monospace", color: COLORS.gray }}>
        Your prompts. Your machine. Your sovereignty.
      </div>
      <div style={{ marginTop: 30, display: "flex", gap: 16, opacity: statsOpacity }}>
        <span style={{ padding: "8px 20px", borderRadius: 10, background: `${COLORS.green}22`, border: `1px solid ${COLORS.green}44`, color: COLORS.green, fontSize: 20, fontFamily: "SF Mono, Menlo, monospace" }}>15 Tools</span>
        <span style={{ padding: "8px 20px", borderRadius: 10, background: `${COLORS.accent}22`, border: `1px solid ${COLORS.accent}44`, color: COLORS.accentGlow, fontSize: 20, fontFamily: "SF Mono, Menlo, monospace" }}>9 Heuristics</span>
        <span style={{ padding: "8px 20px", borderRadius: 10, background: `${COLORS.orange}22`, border: `1px solid ${COLORS.orange}44`, color: COLORS.orange, fontSize: 20, fontFamily: "SF Mono, Menlo, monospace" }}>63 Tests</span>
        <span style={{ padding: "8px 20px", borderRadius: 10, background: `${COLORS.red}22`, border: `1px solid ${COLORS.red}44`, color: COLORS.red, fontSize: 20, fontFamily: "SF Mono, Menlo, monospace" }}>Local Only</span>
      </div>
      <div style={{ marginTop: 40, opacity: ctaOpacity, padding: "14px 44px", borderRadius: 14, background: `linear-gradient(135deg, ${COLORS.accent}, ${COLORS.green})`, fontSize: 26, fontWeight: 700, fontFamily: "SF Pro Display, -apple-system, system-ui, sans-serif", color: COLORS.white, boxShadow: `0 0 ${pulseGlow * 30}px ${COLORS.accent}66` }}>
        cargo build --release
      </div>
      <div style={{ marginTop: 24, opacity: ctaOpacity, fontSize: 20, fontFamily: "SF Mono, Menlo, monospace", color: COLORS.dim }}>
        ExecLayer Inc. — MIT License
      </div>
    </AbsoluteFill>
  );
};

// ── Main composition ──
export const SovereignPromptVideo: React.FC = () => {
  return (
    <AbsoluteFill style={{ background: COLORS.bg }}>
      <Audio src={staticFile("narration.mp3")} />
      <Sequence from={0} durationInFrames={390}>
        <TitleScene />
      </Sequence>
      <Sequence from={390} durationInFrames={360}>
        <ProblemScene />
      </Sequence>
      <Sequence from={750} durationInFrames={330}>
        <AnalysisScene />
      </Sequence>
      <Sequence from={1080} durationInFrames={420}>
        <SolutionScene />
      </Sequence>
      <Sequence from={1500} durationInFrames={330}>
        <ToolsScene />
      </Sequence>
      <Sequence from={1830} durationInFrames={330}>
        <SecurityScene />
      </Sequence>
      <Sequence from={2160} durationInFrames={330}>
        <LearningScene />
      </Sequence>
      <Sequence from={2490} durationInFrames={300}>
        <TechScene />
      </Sequence>
      <Sequence from={2790} durationInFrames={340}>
        <ClosingScene />
      </Sequence>
    </AbsoluteFill>
  );
};
