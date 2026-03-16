function Nav() {
  return (
    <nav className="sc-nav fixed top-0 left-0 right-0 z-50">
      <div className="mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
        <a href="/promptgen" className="flex items-center gap-2">
          <span className="text-lg font-extrabold text-sc-red font-heading tracking-tight">
            SovereignPrompt
          </span>
        </a>
        <div className="flex items-center gap-6">
          <a
            href="#features"
            className="font-cta text-[12px] uppercase tracking-[.1em] text-sc-text-dim transition-colors hover:text-sc-red"
          >
            Features
          </a>
          <a
            href="#how-it-works"
            className="font-cta text-[12px] uppercase tracking-[.1em] text-sc-text-dim transition-colors hover:text-sc-red"
          >
            How It Works
          </a>
          <a
            href="#install"
            className="font-cta text-[12px] uppercase tracking-[.1em] text-sc-text-dim transition-colors hover:text-sc-red"
          >
            Install
          </a>
          <a
            href="https://sovereignclaw.com"
            target="_blank"
            rel="noopener noreferrer"
            className="border border-sc-border-mid px-3 py-1.5 font-cta text-[12px] uppercase tracking-[.1em] text-sc-text-dim transition-colors hover:border-sc-red hover:text-sc-text"
          >
            sovereignclaw.com
          </a>
        </div>
      </div>
    </nav>
  );
}

function Hero() {
  return (
    <section className="relative flex min-h-screen flex-col items-center justify-center px-6 pt-20 text-center">
      {/* Red glow orb */}
      <div className="pointer-events-none absolute top-1/4 h-[500px] w-[500px] bg-sc-red/5 blur-[120px]" />

      <div className="relative z-10">
        <div className="sc-section-tag mb-6 justify-center">
          Part of the SovereignClaw Ecosystem
        </div>

        <h1 className="mb-6 font-heading text-5xl font-extrabold tracking-tight text-sc-text sm:text-7xl">
          Sovereign
          <span className="text-sc-red">Prompt</span>
        </h1>

        <p className="mx-auto mb-4 max-w-2xl text-xl text-sc-text-dim sm:text-2xl">
          Deterministic prompt optimization.
          <br />
          Built in Rust. Runs locally. Zero telemetry.
        </p>

        <p className="mx-auto mb-10 max-w-xl text-base text-sc-text-muted">
          The first MCP-native prompt optimization engine. Strip 30&ndash;75% of
          wasted tokens through 9 heuristic checks, generate 3 strategic
          variants, and get cryptographically signed audit trails.
        </p>

        <div className="flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
          <a href="#install" className="sc-btn-primary">
            Get Started
          </a>
          <a
            href="https://github.com/ExecLayer/sovereign-prompt"
            target="_blank"
            rel="noopener noreferrer"
            className="sc-btn-ghost"
          >
            View on GitHub
          </a>
        </div>

        {/* Stats bar */}
        <div className="mt-16 flex flex-wrap items-center justify-center gap-8 text-sm text-sc-text-muted">
          <div>
            <span className="text-2xl font-bold text-sc-text">15</span>
            <br />
            MCP Tools
          </div>
          <div className="h-8 w-px bg-sc-border" />
          <div>
            <span className="text-2xl font-bold text-sc-text">63</span>
            <br />
            Passing Tests
          </div>
          <div className="h-8 w-px bg-sc-border" />
          <div>
            <span className="text-2xl font-bold text-sc-text">9</span>
            <br />
            Heuristic Checks
          </div>
          <div className="h-8 w-px bg-sc-border" />
          <div>
            <span className="text-2xl font-bold text-sc-red">0</span>
            <br />
            Network Egress
          </div>
        </div>
      </div>
    </section>
  );
}

const features = [
  {
    title: "9 Heuristic Checks",
    description:
      "Redundancy, vagueness, filler words, passive voice, over-qualification, nested clauses, token waste, ambiguity, and structural issues.",
  },
  {
    title: "3 Strategic Variants",
    description:
      "Precision (max specificity), Creative (exploratory framing), and Concise (minimal tokens) — generated deterministically.",
  },
  {
    title: "30-75% Token Savings",
    description:
      "Strips wasted tokens while preserving semantic intent. Measurable savings on every prompt.",
  },
  {
    title: "MCP-Native",
    description:
      "15 tools exposed via Model Context Protocol. Drop into Claude Desktop or Claude Code with zero config friction.",
  },
  {
    title: "Cryptographic Signing",
    description:
      "SHA-256 content hashes + HMAC-SHA256 signatures on every optimization. Full audit trail, tamper-evident.",
  },
  {
    title: "100% Local",
    description:
      "Zero network egress. Zero telemetry. Zero cloud dependencies. Your prompts never leave your machine.",
  },
];

const featureIcons = [
  <svg key="0" className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}><path strokeLinecap="round" strokeLinejoin="round" d="M9.75 3.75a6 6 0 110 12 6 6 0 010-12zM16.5 16.5L21 21" /></svg>,
  <svg key="1" className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}><path strokeLinecap="round" strokeLinejoin="round" d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.455 2.456L21.75 6l-1.036.259a3.375 3.375 0 00-2.455 2.456z" /></svg>,
  <svg key="2" className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}><path strokeLinecap="round" strokeLinejoin="round" d="M3.75 13.5l10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75z" /></svg>,
  <svg key="3" className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}><path strokeLinecap="round" strokeLinejoin="round" d="M14.25 6.087c0-.355.186-.676.401-.959.221-.29.349-.634.349-1.003 0-1.036-1.007-1.875-2.25-1.875S10.5 3.089 10.5 4.125c0 .369.128.713.349 1.003.215.283.401.604.401.959V6a.75.75 0 01-.75.75H7.5a.75.75 0 00-.75.75v2.25c0 .414-.336.75-.75.75h-.087c-.355 0-.676.186-.959.401-.29.221-.634.349-1.003.349-1.036 0-1.875-1.007-1.875-2.25s.839-2.25 1.875-2.25c.369 0 .713.128 1.003.349.283.215.604.401.959.401H6a.75.75 0 01.75-.75h3a.75.75 0 00.75-.75V4.125" /></svg>,
  <svg key="4" className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}><path strokeLinecap="round" strokeLinejoin="round" d="M16.5 10.5V6.75a4.5 4.5 0 10-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 002.25-2.25v-6.75a2.25 2.25 0 00-2.25-2.25H6.75a2.25 2.25 0 00-2.25 2.25v6.75a2.25 2.25 0 002.25 2.25z" /></svg>,
  <svg key="5" className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}><path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" /></svg>,
];

function Features() {
  return (
    <section id="features" className="px-6 py-24">
      <div className="mx-auto max-w-6xl">
        <div className="sc-section-tag justify-center">Features</div>
        <h2 className="mb-4 text-center font-heading text-3xl font-extrabold text-sc-text sm:text-4xl">
          Built for precision
        </h2>
        <p className="mx-auto mb-16 max-w-xl text-center text-sc-text-muted">
          Every feature designed around one goal: make your prompts measurably
          better without leaving your machine.
        </p>

        <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {features.map((feature, i) => (
            <div
              key={feature.title}
              className="sc-card group"
            >
              <div className="mb-4 flex h-10 w-10 items-center justify-center border border-sc-border bg-sc-red/5 text-sc-red">
                {featureIcons[i]}
              </div>
              <h3 className="mb-2 font-heading text-lg font-bold text-sc-text">
                {feature.title}
              </h3>
              <p className="text-sm leading-relaxed text-sc-text-dim">
                {feature.description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function HowItWorks() {
  const steps = [
    {
      step: "01",
      label: "Input",
      detail: "Raw prompt submitted via MCP tool call",
    },
    {
      step: "02",
      label: "Analyze",
      detail: "9 heuristic checks score the prompt",
    },
    {
      step: "03",
      label: "Optimize",
      detail: "Strip waste, preserve semantic intent",
    },
    {
      step: "04",
      label: "Generate",
      detail: "3 strategic variants (Precision, Creative, Concise)",
    },
    {
      step: "05",
      label: "Sign",
      detail: "SHA-256 hash + HMAC-SHA256 signature",
    },
    {
      step: "06",
      label: "Return",
      detail: "Optimized prompt + audit receipt via MCP",
    },
  ];

  return (
    <section id="how-it-works" className="px-6 py-24">
      <div className="mx-auto max-w-4xl">
        <div className="sc-section-tag justify-center">Pipeline</div>
        <h2 className="mb-4 text-center font-heading text-3xl font-extrabold text-sc-text sm:text-4xl">
          How it works
        </h2>
        <p className="mx-auto mb-16 max-w-xl text-center text-sc-text-muted">
          A deterministic pipeline from raw prompt to signed, optimized output.
        </p>

        <div className="relative">
          {/* Connecting line */}
          <div className="absolute left-[23px] top-0 hidden h-full w-px bg-gradient-to-b from-sc-red/50 via-sc-red/20 to-transparent sm:block" />

          <div className="space-y-8">
            {steps.map((s) => (
              <div key={s.step} className="flex items-start gap-6">
                <div className="flex h-12 w-12 shrink-0 items-center justify-center border border-sc-red/30 bg-sc-red/5 font-mono text-sm font-bold text-sc-red">
                  {s.step}
                </div>
                <div className="pt-1">
                  <h3 className="font-heading text-lg font-bold text-sc-text">
                    {s.label}
                  </h3>
                  <p className="text-sm text-sc-text-dim">{s.detail}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

function Install() {
  return (
    <section id="install" className="px-6 py-24">
      <div className="mx-auto max-w-4xl">
        <div className="sc-section-tag justify-center">Quick Start</div>
        <h2 className="mb-4 text-center font-heading text-3xl font-extrabold text-sc-text sm:text-4xl">
          Get started in minutes
        </h2>
        <p className="mx-auto mb-16 max-w-xl text-center text-sc-text-muted">
          Build from source, then drop into Claude Desktop or Claude Code.
        </p>

        {/* Build */}
        <div className="mb-10">
          <h3 className="mb-3 font-mono text-sm uppercase tracking-wider text-sc-red">
            1. Build
          </h3>
          <div className="sc-code-block">
            <pre className="font-mono text-sm text-sc-text-dim">
              <code>{`git clone https://github.com/ExecLayer/sovereign-prompt.git
cd sovereign-prompt
cargo build --release`}</code>
            </pre>
          </div>
        </div>

        {/* Claude Desktop */}
        <div className="mb-10">
          <h3 className="mb-3 font-mono text-sm uppercase tracking-wider text-sc-red">
            2. Claude Desktop Config
          </h3>
          <p className="mb-3 text-sm text-sc-text-muted">
            Add to{" "}
            <code className="border border-sc-border bg-sc-iron px-1.5 py-0.5 text-xs text-sc-text-dim">
              claude_desktop_config.json
            </code>
          </p>
          <div className="sc-code-block">
            <pre className="font-mono text-sm text-sc-text-dim">
              <code>{`{
  "mcpServers": {
    "sovereign-prompt": {
      "command": "/path/to/sovereign-prompt/target/release/sovereign-prompt",
      "args": ["--mcp"]
    }
  }
}`}</code>
            </pre>
          </div>
        </div>

        {/* Claude Code */}
        <div className="mb-10">
          <h3 className="mb-3 font-mono text-sm uppercase tracking-wider text-sc-red">
            3. Claude Code
          </h3>
          <div className="sc-code-block">
            <pre className="font-mono text-sm text-sc-text-dim">
              <code>{`claude mcp add sovereign-prompt /path/to/sovereign-prompt/target/release/sovereign-prompt -- --mcp`}</code>
            </pre>
          </div>
        </div>

        {/* Available tools */}
        <div>
          <h3 className="mb-3 font-mono text-sm uppercase tracking-wider text-sc-red">
            Available MCP Tools
          </h3>
          <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
            {[
              "optimize_prompt",
              "analyze_prompt",
              "generate_variants",
              "batch_optimize",
              "get_optimization_history",
              "compare_prompts",
              "sign_prompt",
              "verify_signature",
              "get_heuristic_details",
              "export_audit_trail",
              "get_token_stats",
              "clear_history",
              "get_config",
              "set_config",
              "health_check",
            ].map((tool) => (
              <div
                key={tool}
                className="border border-sc-border bg-transparent px-3 py-2 font-mono text-xs text-sc-text-dim"
              >
                {tool}
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

function Footer() {
  return (
    <footer className="border-t border-sc-border px-6 py-12">
      <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-4 sm:flex-row">
        <div className="text-sm text-sc-text-muted">
          SovereignPrompt &mdash;{" "}
          <a
            href="https://execlayer.com"
            target="_blank"
            rel="noopener noreferrer"
            className="text-sc-text-dim transition-colors hover:text-sc-red"
          >
            ExecLayer Inc.
          </a>
        </div>
        <div className="flex gap-6 text-sm text-sc-text-muted">
          <a
            href="https://sovereignclaw.com"
            target="_blank"
            rel="noopener noreferrer"
            className="transition-colors hover:text-sc-red"
          >
            SovereignClaw
          </a>
          <a
            href="https://github.com/ExecLayer/sovereign-prompt"
            target="_blank"
            rel="noopener noreferrer"
            className="transition-colors hover:text-sc-red"
          >
            GitHub
          </a>
        </div>
      </div>
    </footer>
  );
}

export default function Home() {
  return (
    <main className="min-h-screen">
      <Nav />
      <Hero />
      <Features />
      <HowItWorks />
      <Install />
      <Footer />
    </main>
  );
}
