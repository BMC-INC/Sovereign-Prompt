import React from "react";
import {
  AbsoluteFill,
  Easing,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";

const COLORS = {
  bg: "#071019",
  card: "#102130",
  edge: "#29445f",
  text: "#e7f3ff",
  muted: "#9ab6d3",
  accentA: "#22c3ff",
  accentB: "#65f0a5",
  accentC: "#ffc66d",
  accentD: "#f9865f",
};

const cardStyle: React.CSSProperties = {
  borderRadius: 16,
  border: `1px solid ${COLORS.edge}`,
  background: "linear-gradient(155deg, #132437, #0c1a29)",
  boxShadow: "0 24px 55px rgba(0, 8, 18, 0.45)",
};

const Background: React.FC = () => (
  <>
    <div
      style={{
        position: "absolute",
        inset: 0,
        background:
          "radial-gradient(circle at 20% 20%, #134065 0%, transparent 36%), radial-gradient(circle at 78% 85%, #1f4930 0%, transparent 28%), linear-gradient(180deg, #04090f 0%, #091420 100%)",
      }}
    />
    <div
      style={{
        position: "absolute",
        inset: 0,
        opacity: 0.2,
        backgroundImage:
          "linear-gradient(#33516d22 1px, transparent 1px), linear-gradient(90deg, #33516d22 1px, transparent 1px)",
        backgroundSize: "38px 38px",
      }}
    />
  </>
);

const Node: React.FC<{
  label: string;
  subtitle: string;
  color: string;
  x: number;
  y: number;
  delay: number;
}> = ({ label, subtitle, color, x, y, delay }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const progress = spring({
    frame: frame - delay,
    fps,
    config: { damping: 200 },
  });
  const opacity = interpolate(frame, [delay, delay + 10], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <div
      style={{
        ...cardStyle,
        position: "absolute",
        left: x,
        top: y,
        width: 260,
        padding: "16px 18px",
        opacity,
        transform: `translateY(${(1 - progress) * 24}px) scale(${0.96 + progress * 0.04})`,
        borderColor: `${color}55`,
      }}
    >
      <div
        style={{
          color,
          fontSize: 14,
          fontWeight: 700,
          letterSpacing: 0.7,
          textTransform: "uppercase",
          marginBottom: 6,
          fontFamily: "'IBM Plex Sans', 'Segoe UI', sans-serif",
        }}
      >
        {label}
      </div>
      <div
        style={{
          color: COLORS.text,
          fontSize: 17,
          lineHeight: 1.35,
          fontWeight: 600,
          fontFamily: "'IBM Plex Sans', 'Segoe UI', sans-serif",
        }}
      >
        {subtitle}
      </div>
    </div>
  );
};

const Arrow: React.FC<{
  x: number;
  y: number;
  w: number;
  delay: number;
  color: string;
}> = ({ x, y, w, delay, color }) => {
  const frame = useCurrentFrame();
  const growth = interpolate(frame, [delay, delay + 20], [0, w], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });
  const opacity = interpolate(frame, [delay, delay + 10], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  return (
    <div style={{ position: "absolute", left: x, top: y, width: w, opacity }}>
      <div
        style={{
          width: growth,
          height: 3,
          borderRadius: 3,
          background: `linear-gradient(90deg, ${color}, ${COLORS.text})`,
          boxShadow: `0 0 15px ${color}88`,
        }}
      />
      <div
        style={{
          position: "absolute",
          right: 0,
          top: -6,
          width: 0,
          height: 0,
          borderTop: "7px solid transparent",
          borderBottom: "7px solid transparent",
          borderLeft: `12px solid ${COLORS.text}`,
        }}
      />
    </div>
  );
};

export const HowItWorksGraphic: React.FC = () => {
  const frame = useCurrentFrame();
  const titleOpacity = interpolate(frame, [0, 18], [0, 1], {
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center" }}>
      <Background />
      <div
        style={{
          position: "absolute",
          top: 30,
          color: COLORS.text,
          fontSize: 42,
          fontWeight: 800,
          letterSpacing: -1,
          opacity: titleOpacity,
          fontFamily: "'Sora', 'IBM Plex Sans', sans-serif",
        }}
      >
        How SovereignPrompt Works
      </div>

      <Node
        label="Client Request"
        subtitle="Incoming prompt and user context"
        color={COLORS.accentA}
        x={30}
        y={208}
        delay={12}
      />
      <Arrow x={280} y={273} w={80} delay={24} color={COLORS.accentA} />
      <Node
        label="Analyzer"
        subtitle="9 heuristics identify waste and risk"
        color={COLORS.accentC}
        x={360}
        y={120}
        delay={34}
      />
      <Arrow x={490} y={278} w={0} delay={0} color={COLORS.accentA} />
      <Node
        label="Optimizer + Templates"
        subtitle="Domain-specific prompt shaping"
        color={COLORS.accentB}
        x={360}
        y={296}
        delay={48}
      />
      <Arrow x={610} y={170} w={80} delay={62} color={COLORS.accentC} />
      <Arrow x={610} y={344} w={80} delay={66} color={COLORS.accentB} />
      <Node
        label="Tokenizer"
        subtitle="Counts across cl100k/o200k/p50k/r50k"
        color={COLORS.accentD}
        x={690}
        y={208}
        delay={76}
      />
      <Arrow x={810} y={273} w={80} delay={90} color={COLORS.accentD} />
      <Node
        label="Persistence + Output"
        subtitle="Stored analytics + optimized response"
        color={COLORS.accentA}
        x={890}
        y={208}
        delay={102}
      />
    </AbsoluteFill>
  );
};

export const OptimizationPipelineGraphic: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const titleOpacity = interpolate(frame, [0, 14], [0, 1], {
    extrapolateRight: "clamp",
  });
  const pulse = 0.5 + 0.5 * Math.sin(frame / 12);
  const stages = [
    {
      title: "Input Prompt",
      body: "User request enters pipeline",
      color: COLORS.accentA,
      delay: 12,
    },
    {
      title: "Heuristic Analysis",
      body: "Flags clarity, security, format issues",
      color: COLORS.accentC,
      delay: 34,
    },
    {
      title: "Template Injection",
      body: "Applies per-domain constraints",
      color: COLORS.accentB,
      delay: 56,
    },
    {
      title: "Refine + Variants",
      body: "Builds precision / creative / concise",
      color: COLORS.accentD,
      delay: 78,
    },
    {
      title: "Model Token Matrix",
      body: "Counts with cl100k/o200k/p50k/r50k",
      color: COLORS.accentA,
      delay: 100,
    },
  ];

  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center" }}>
      <Background />
      <div
        style={{
          position: "absolute",
          top: 28,
          color: COLORS.text,
          fontSize: 40,
          fontWeight: 800,
          letterSpacing: -1,
          opacity: titleOpacity,
          fontFamily: "'Sora', 'IBM Plex Sans', sans-serif",
        }}
      >
        Optimization Pipeline
      </div>

      {stages.map((stage, i) => {
        const x = 36 + i * 184;
        const y = 208;
        const progress = spring({
          frame: frame - stage.delay,
          fps,
          config: { damping: 200 },
        });
        const opacity = interpolate(frame, [stage.delay, stage.delay + 10], [0, 1], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
        });

        return (
          <React.Fragment key={stage.title}>
            <div
              style={{
                ...cardStyle,
                position: "absolute",
                left: x,
                top: y,
                width: 176,
                height: 170,
                padding: "14px 14px",
                borderColor: `${stage.color}66`,
                opacity,
                transform: `translateY(${(1 - progress) * 28}px)`,
                boxShadow:
                  i === 4
                    ? `0 0 ${16 + pulse * 14}px ${stage.color}55`
                    : cardStyle.boxShadow,
              }}
            >
              <div
                style={{
                  color: stage.color,
                  fontSize: 13,
                  fontWeight: 800,
                  letterSpacing: 0.5,
                  textTransform: "uppercase",
                  marginBottom: 8,
                  fontFamily: "'IBM Plex Sans', 'Segoe UI', sans-serif",
                }}
              >
                {stage.title}
              </div>
              <div
                style={{
                  color: COLORS.text,
                  fontSize: 16,
                  lineHeight: 1.35,
                  fontWeight: 600,
                  fontFamily: "'IBM Plex Sans', 'Segoe UI', sans-serif",
                }}
              >
                {stage.body}
              </div>
              <div
                style={{
                  position: "absolute",
                  left: 14,
                  right: 14,
                  bottom: 12,
                  height: 6,
                  borderRadius: 6,
                  background: "#12263a",
                  overflow: "hidden",
                }}
              >
                <div
                  style={{
                    width: `${Math.min(
                      100,
                      Math.max(0, ((frame - stage.delay) / 38) * 100)
                    )}%`,
                    height: "100%",
                    background: `linear-gradient(90deg, ${stage.color}, ${COLORS.text})`,
                  }}
                />
              </div>
            </div>
            {i < stages.length - 1 ? (
              <Arrow
                x={x + 170}
                y={287}
                w={24}
                delay={stage.delay + 12}
                color={stage.color}
              />
            ) : null}
          </React.Fragment>
        );
      })}

      <div
        style={{
          position: "absolute",
          bottom: 42,
          color: COLORS.muted,
          fontSize: 19,
          fontFamily: "'IBM Plex Sans', 'Segoe UI', sans-serif",
          letterSpacing: 0.2,
        }}
      >
        Deterministic pipeline with domain strategy + model-aware metrics.
      </div>
    </AbsoluteFill>
  );
};
